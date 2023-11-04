use std::{
    ffi::OsString,
    fmt::Display,
    io::{self, Write},
    path::Path,
    process::Command,
    time::Duration,
};

use error_stack::{bail, ResultExt};
use rustshop_env::{AccountCfg, Env};
use tempfile::NamedTempFile;
use tracing::{info, trace, warn};

const CF_BOOTSTRAP_TERRAFORM_YAML: &str = include_str!("./bootstrap/cf-bootstrap-terraform.yaml");
const CF_BOOTSTRAP_CLOUDTRAIL_YAML: &str = include_str!("./bootstrap/cf-bootstrap-cloudtrail.yaml");
const CF_BOOTSTRAP_KOPS_YAML: &str = include_str!("./bootstrap/cf-bootstrap-kops.yaml");

use crate::{
    aws_api::{self, Aws, AwsError, AwsResult},
    opts::EmailBootstrapOpts,
    AppError, AppResult,
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
        .split_once('@')
        .ok_or(AwsError::Io)
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
    let cf_bootstrap_terraform_file =
        create_cf_bootstrap_file(CF_BOOTSTRAP_TERRAFORM_YAML).change_context(AwsError::Io)?;
    let cf_bootstrap_cloudtrail_file =
        create_cf_bootstrap_file(CF_BOOTSTRAP_CLOUDTRAIL_YAML).change_context(AwsError::Io)?;
    let cf_bootstrap_kops_file =
        create_cf_bootstrap_file(CF_BOOTSTRAP_KOPS_YAML).change_context(AwsError::Io)?;

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
                    Err(AwsError::WrongResponse).attach_printable_lazy(
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
        let role = aws.assume_account_root_role(account)?;
        aws.with_creds(role.credentials.clone())
    };

    deploy_stack(
        &aws,
        &account.name,
        "cloudtrail",
        cf_bootstrap_cloudtrail_file.path(),
    )?;
    deploy_stack(
        &aws,
        &account.name,
        "terraform",
        cf_bootstrap_terraform_file.path(),
    )?;

    deploy_stack(&aws, &account.name, "kops", cf_bootstrap_kops_file.path())?;

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

fn retry<T, E>(f: impl Fn() -> std::result::Result<T, E>) -> std::result::Result<T, E>
where
    E: Display,
{
    loop {
        match (f)() {
            Ok(o) => return Ok(o),
            Err(e) => warn!("Will retry on error: {e}"),
        }
    }
}

pub fn bootstrap_cluster(
    account_name: Option<String>,
    cluster_name: Option<String>,
    create_hosted_zone: bool,
    dns_ready: bool,
    minimal: bool,
    other_args: &[OsString],
) -> AppResult<()> {
    let mut env = Env::load().change_context(AppError::Other)?;

    let account_name = if let Some(account_name) = account_name {
        account_name
    } else {
        env.get_context_account()
            .change_context(AppError::Other)?
            .account
            .expect("get_account_context took care of it")
            .0
    };

    // default account name if no name provided
    let cluster_name = cluster_name.unwrap_or_else(|| account_name.clone());

    let account_cfg: AccountCfg = env
        .get_account_ref(&account_name)
        .change_context(AppError::Other)?
        .into();

    let cluster_cfg = if let Some(cluster_cfg) = env
        .get_account_ref(&account_name)
        .change_context(AppError::Other)?
        .get_cluster_ref_opt(&cluster_name)
        .change_context(AppError::Other)?
    {
        cluster_cfg.shop.clone()
    } else {
        env.add_cluster(&account_name, &cluster_name)
            .change_context(AppError::Other)?
    };

    let aws = aws_api::Aws::new(
        Some(account_cfg.user.aws_profile.clone()),
        account_cfg.shop.bootstrap_aws_region.clone(),
    );

    // by using the same caller id, we won't create the zone multiple times
    let caller_id = Aws::random_caller_id();

    let zone = loop {
        info!(name = cluster_cfg.domain, "Checking if hosted zone exists");
        if let Some(zone) = aws
            .list_hosted_zones()
            .change_context(AppError::Other)?
            .into_iter()
            .filter(|zone| {
                zone.name == cluster_cfg.domain /* kops cluster name is the cluster domain */
                || zone.name == format!("{}.", cluster_cfg.domain)
            } /* Note: FQDN suffixed with `.` */)
            .next()
        {
            info!(
                "Zone name: {} id: {} already exist with DNS names",
                zone.name, zone.id
            );
            break zone;
        } else if !create_hosted_zone {
            Err(AppError::Other).attach_printable_lazy(|| {
                "Existing zone not detected. Rerun with `--create-hosted-zone` to create"
            })?;
        } else {
            retry(|| {
                info!("Creating zone name: {}", cluster_cfg.domain);
                aws.create_hosted_zone(&cluster_cfg.domain, Some(caller_id.clone()))
                    .change_context(AppError::Other)
            })?
        }
    };

    if !dns_ready {
        let zone_details =
            retry(|| aws.get_hosted_zone(&zone.id)).change_context(AppError::Other)?;
        info!(
            "Zone ready. Configure NS records for domain ({}) in root domain ({}) to point at: {:?}",
            cluster_cfg.domain,
            env.shop_cfg().domain,
            zone_details.delegation_set.name_servers
        );
        info!(
            "Verify with `dig {}`) and restart bootstrap with `--dns-ready`",
            cluster_cfg.domain
        );
        return Ok(());
    }

    let mut cmd = Command::new("kops");

    super::wrap::set_kops_envs_on(&account_cfg.shop, &cluster_cfg, &mut cmd)
        .change_context(AppError::Other)?;

    cmd.args([
        "create",
        "cluster",
        "--cloud",
        "aws",
        "--zones",
        &format!("{}a", account_cfg.shop.bootstrap_aws_region),
        &format!(
            "--discovery-store=s3://{}-bootstrap-kops-oidc-public/{}/discovery",
            account_cfg.shop.bootstrap_name, cluster_cfg.domain
        ),
    ]);

    if minimal {
        cmd.args([
            "--master-count",
            "1",
            "--master-size",
            "t3a.small",
            "--master-volume-size",
            "8",
            "--node-count",
            "1",
            "--node-size",
            "t3a.small",
            "--node-volume-size",
            "8",
        ]);
    }

    cmd.args(other_args);

    trace!("Run: {cmd:?}");
    let status = cmd.output().change_context(AppError::Other)?;

    if !status.status.success() {
        bail!(AppError::CommandFailed {
            stderr: String::from_utf8_lossy(&status.stderr).to_string()
        })
    }

    info!("Cluster created with `kops`. Use `kops edit cluster` to tune, and `kups update cluster --yes` to deploy");
    Ok(())
}
