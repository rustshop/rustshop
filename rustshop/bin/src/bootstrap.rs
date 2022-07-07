use std::{
    io::{self, Write},
    path::Path,
    time::Duration,
};

use error_stack::{IntoReport, ResultExt};
use tempfile::NamedTempFile;
use tracing::info;

const CF_BOOTSTRAP_TERRAFORM_YAML: &'static str =
    include_str!("./bootstrap/cf-bootstrap-terraform.yaml");
const CF_BOOTSTRAP_CLOUDTRAIL_YAML: &'static str =
    include_str!("./bootstrap/cf-bootstrap-cloudtrail.yaml");
const CF_BOOTSTRAP_KOPS_YAML: &'static str = include_str!("./bootstrap/cf-bootstrap-kops.yaml");

use crate::{
    aws_api::{self, Aws, AwsError, AwsResult},
    opts::EmailBootstrapOpts,
};

struct EmailParts {
    user: String,
    domain: String,
}

impl EmailParts {
    fn generate_account_email(
        &self,
        account_name: &str,
        email_opts: &EmailBootstrapOpts,
    ) -> String {
        format!(
            "{}+{}{}{}@{}",
            self.user,
            email_opts.email_label_prefix,
            account_name,
            email_opts.email_label_suffix,
            self.domain
        )
    }
}

fn parse_email(email: &str) -> AwsResult<EmailParts> {
    let (user, domain) = email
        .split_once("@")
        .ok_or(AwsError::Io)
        .report()
        .attach_printable_lazy(|| format!("Email does not contain `@`: {email}"))?;
    Ok(EmailParts {
        user: user.to_owned(),
        domain: domain.to_owned(),
    })
}

fn create_cf_bootstrap_file(content: &str) -> std::result::Result<NamedTempFile, io::Error> {
    let mut file = NamedTempFile::new()?;

    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(file)
}

pub(crate) fn bootstrap_org(profile: &str) -> AwsResult<()> {
    let aws = aws_api::Aws::new(
        Some(profile.to_string()),
        "us-east-1".into(), /* doesn't matter, nothing is being created here */
    );

    let organization_details = aws.create_or_get_organization()?;
    info!("Your organization: {:?}", organization_details);

    Ok(())
}

pub fn get_root_account_id(profile: Option<&str>) -> AwsResult<String> {
    let aws = aws_api::Aws::new(
        profile.map(ToString::to_string),
        "us-east-1".into(), /* doesn't matter */
    );
    Ok(aws.get_caller_identity()?.account)
}

pub fn bootstrap_account(
    shop_name: &str,
    account_suffix: &str,
    profile: &str,
    aws_region: &str,
    email_opts: &EmailBootstrapOpts,
) -> AwsResult<String> {
    let cf_bootstrap_terraform_file = create_cf_bootstrap_file(CF_BOOTSTRAP_TERRAFORM_YAML)
        .report()
        .change_context(AwsError::Io)?;
    let cf_bootstrap_cloudtrail_file = create_cf_bootstrap_file(CF_BOOTSTRAP_CLOUDTRAIL_YAML)
        .report()
        .change_context(AwsError::Io)?;
    let cf_bootstrap_kops_file = create_cf_bootstrap_file(CF_BOOTSTRAP_KOPS_YAML)
        .report()
        .change_context(AwsError::Io)?;

    let aws = aws_api::Aws::new(Some(profile.to_owned()), aws_region.to_string());

    let root_account_id = aws.get_caller_identity()?.account;

    let mut existing_accounts = aws.list_existing_accouns()?;
    let full_account_name = format!("{}-{}", shop_name, account_suffix);

    if existing_accounts
        .iter()
        .any(|existing| existing.name == full_account_name)
    {
        tracing::info!("Account {full_account_name} already exists");
    } else {
        let base_email = parse_email(&email_opts.email)?;
        let email = base_email.generate_account_email(account_suffix, email_opts);

        info!("Creating {full_account_name}; email: {email}");

        aws.create_account(&full_account_name, &email)?;
    }

    loop {
        existing_accounts = aws.list_existing_accouns()?;

        for account in &existing_accounts {
            match account.status {
                aws_api::Status::Active => {}
                aws_api::Status::InProgress => {
                    info!("Account {} still being created", account.name);
                    std::thread::sleep(Duration::from_secs(10));
                    continue;
                }
                aws_api::Status::Other => {
                    Err(AwsError::WrongResponse).report().attach_printable_lazy(
                        ||
                        format!("Account {} in unknown status. Correct manually in AWS console and try again.", account.name)
                    )?;
                }
            }
        }

        break;
    }

    info!("Account ready");

    let account = existing_accounts
        .iter()
        .find(|acc| acc.name == full_account_name)
        .ok_or(AwsError::WrongResponse)?;

    let aws = if account.id == root_account_id {
        aws
    } else {
        let role = aws.assume_account_root_role(&account)?;
        aws.with_creds(role.credentials.clone())
    };

    deploy_stack(
        &aws,
        &account.name,
        "cloudtrail",
        &cf_bootstrap_cloudtrail_file.path(),
    )?;
    deploy_stack(
        &aws,
        &account.name,
        "terraform",
        &cf_bootstrap_terraform_file.path(),
    )?;

    deploy_stack(&aws, &account.name, "kops", &cf_bootstrap_kops_file.path())?;

    Ok(account.id.clone())
}

fn deploy_stack(
    aws: &Aws,
    full_account_name: &str,
    stack_name: &str,
    file: &Path,
) -> AwsResult<()> {
    // Prefix is required due to S3 Buckets having global namespace
    let full_stack_name = format!("{full_account_name}-bootstrap-{stack_name}");
    info!("Deploying CF Stack {full_stack_name}");
    aws.deploy_cf(&full_stack_name, file)?;
    Ok(())
}
