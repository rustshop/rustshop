use std::process::Command;

use crate::opts::Opts;
use color_eyre::Result;
use eyre::{bail, eyre};
use serde::{de::DeserializeOwned, Deserialize};
use tracing::{debug, trace};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum Status {
    Active,
    InProgress,
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Account {
    pub id: String,
    pub arn: String,
    pub email: String,
    pub name: String,
    pub status: Status,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Organization {
    pub id: String,
    pub arn: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct AccountList {
    pub accounts: Vec<Account>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct OrganizationDetails {
    pub organization: Organization,
}

fn run_cmd<T>(opts: &Opts, args: &[&str]) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let output = run_cmd_raw(opts, args)?;
    Ok(if let Some(output) = output {
        Some(serde_json::from_slice(&output)?)
    } else {
        None
    })
}

fn run_cmd_raw(opts: &Opts, args: &[&str]) -> Result<Option<Vec<u8>>> {
    let output = {
        let mut cmd = Command::new("aws");

        let cmd = cmd.arg("--profile").arg(&opts.profile).args(args);

        trace!("Running: {:?}", cmd);
        cmd.output()?
    };

    trace!("Status code: {:?}", output.status.code());
    debug!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    debug!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    // 254 seems to indicate non-fatal error, like
    if output.status.code().unwrap_or(-1) == 254 {
        return Ok(None);
    }

    if !output.status.success() {
        bail!(
            "Command failed with:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(Some(output.stdout))
}

pub fn create_or_get_organization(opts: &Opts) -> Result<Organization> {
    Ok(
        match run_cmd::<OrganizationDetails>(
            &opts,
            &[
                "organizations",
                "create-organization",
                "--feature-set",
                "ALL",
            ],
        )? {
            Some(org) => org.organization,
            None => {
                run_cmd::<OrganizationDetails>(&opts, &["organizations", "describe-organization"])?
                    .ok_or_else(|| eyre!("Failed to create/fetch existing organization details"))?
                    .organization
            }
        },
    )
}

pub(crate) fn list_existing_accouns(opts: &Opts) -> Result<Vec<Account>> {
    Ok(
        run_cmd::<AccountList>(&opts, &["organizations", "list-accounts"])?
            .ok_or_else(|| eyre!("Could not get list of accounts"))?
            .accounts,
    )
}

pub(crate) fn create_account(opts: &Opts, account_name: &str, email: &str) -> Result<String> {
    Ok(String::from_utf8(
        run_cmd_raw(
            &opts,
            &[
                "organizations",
                "create-account",
                "--email",
                &email,
                "--account-name",
                account_name,
            ],
        )?
        .ok_or_else(|| eyre!("Failed to create account: {account_name}"))?,
    )?)
}
