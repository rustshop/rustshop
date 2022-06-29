#![feature(exit_status_error)]

use std::{io::Write, time::Duration};

use aws_api::Aws;
use color_eyre::{eyre, Result};
use eyre::{bail, eyre};
use tempfile::NamedTempFile;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod aws_api;
mod opts;

const CF_BOOTSTRAP_YAML: &'static str = include_str!("./cf-bootstrap.yaml");

use opts::Opts;

struct EmailParts {
    user: String,
    domain: String,
}

impl EmailParts {
    fn generate_account_email(&self, account_name: &str, opts: &Opts) -> String {
        format!(
            "{}+{}{}{}@{}",
            self.user, opts.email_label_prefix, account_name, opts.email_label_suffix, self.domain
        )
    }
}

fn parse_email(email: &str) -> Result<EmailParts> {
    let (user, domain) = email
        .split_once("@")
        .ok_or_else(|| eyre!("Email does not contain `@`: {email}"))?;
    Ok(EmailParts {
        user: user.to_owned(),
        domain: domain.to_owned(),
    })
}

fn create_cf_bootstrap_file() -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;

    file.write_all(CF_BOOTSTRAP_YAML.as_bytes())?;
    file.flush()?;

    Ok(file)
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "aws_bootstrap=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    color_eyre::install()?;
    let opts = opts::parse();

    tracing::debug!("Opts: {opts:?}");

    let cf_bootstrap_file = create_cf_bootstrap_file()?;
    let aws = aws_api::Aws::new(opts.profile.clone());

    let root_account_id = aws.get_caller_identity()?.account;

    tracing::debug!("Opts: {opts:?}");

    let organization_details = aws.create_or_get_organization()?;
    info!("Your organization: {:?}", organization_details);

    let base_email = parse_email(&opts.email)?;

    let mut existing_accounts = aws.list_existing_accouns()?;

    let root_account = existing_accounts
        .iter()
        .find(|acc| acc.id == root_account_id)
        .ok_or_else(|| {
            eyre!("Could not find an account matching root account id: {root_account_id}")
        })?
        .to_owned();

    for account_name_suffix in &opts.accounts {
        let full_account_name = format!("{}-{}", opts.base_account_name, account_name_suffix);
        if existing_accounts
            .iter()
            .any(|existing| existing.name == full_account_name)
        {
            tracing::info!("Account {full_account_name} already exists");
        } else {
            let email = base_email.generate_account_email(account_name_suffix, &opts);
            tracing::info!("Creating {full_account_name}; email: {email}");

            aws.create_account(&full_account_name, &email)?;
        }
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
                    bail!("Account {} in unknown status. Correct manually in AWS console and try again.", account.name);
                }
            }
        }

        break;
    }

    deploy_cf_bootrap(&aws, &root_account.id, cf_bootstrap_file.path())?;

    for account_name_suffix in &opts.accounts {
        let full_account_name = format!("{}-{}", opts.base_account_name, account_name_suffix);
        let account = existing_accounts
            .iter()
            .find(|acc| acc.name == full_account_name)
            .ok_or_else(|| eyre!("Could not look up account {full_account_name} ID"))?;
        let role = aws.assume_account_root_role(&account)?;
        deploy_cf_bootrap(
            &aws.with_creds(role.credentials.clone()),
            &account.name,
            cf_bootstrap_file.path(),
        )?;
    }

    Ok(())
}

fn deploy_cf_bootrap(_aws: &Aws, _account_full_name: &str, _path: &std::path::Path) -> Result<()> {
    Ok(())
}
