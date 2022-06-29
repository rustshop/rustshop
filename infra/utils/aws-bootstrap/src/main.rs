#![feature(exit_status_error)]

use std::time::Duration;

use color_eyre::{eyre, Result};
use eyre::{bail, eyre};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod aws_api;
mod opts;

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

    let organization_details = aws_api::create_or_get_organization(&opts)?;
    info!("Your organization: {:?}", organization_details);

    let base_email = parse_email(&opts.email)?;

    let mut existing_accounts = aws_api::list_existing_accouns(&opts)?;

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

            aws_api::create_account(&opts, &full_account_name, &email)?;
        }
    }

    loop {
        existing_accounts = aws_api::list_existing_accouns(&opts)?;

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

    Ok(())
}
