#![feature(exit_status_error)]

use color_eyre::{eyre, Result};
use eyre::eyre;
use std::process::{Command, Stdio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod list_accounts;
mod opts;

use opts::Opts;
use serde::de::DeserializeOwned;

fn run_aws_cmd<T>(opts: &Opts, args: &[&str]) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_slice(&run_aws_cmd_raw(opts, args)?)?)
}

fn run_aws_cmd_raw(opts: &Opts, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("aws")
        .arg("--profile")
        .arg(&opts.profile)
        .args(args)
        .stderr(Stdio::inherit())
        .output()?;

    tracing::debug!("Output: {}", String::from_utf8_lossy(&output.stdout));
    output.status.exit_ok()?;

    Ok(output.stdout)
}

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

    let base_email = parse_email(&opts.email)?;

    let existing_accounts =
        run_aws_cmd::<list_accounts::Output>(&opts, &["organizations", "list-accounts"])?.accounts;

    for account_name in &opts.accounts {
        if !existing_accounts
            .iter()
            .any(|existing| &existing.name == account_name)
        {
            tracing::info!("Account {account_name} already exists");
        } else {
            let email = base_email.generate_account_email(account_name, &opts);
            tracing::info!("Creating {account_name}; email: {email}");
            run_aws_cmd_raw(
                &opts,
                &[
                    "aws",
                    "organizations",
                    "create-account",
                    "--email",
                    &email,
                    "--account-name",
                    account_name,
                ],
            )?;
        }
    }

    Ok(())
}
