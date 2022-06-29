use std::process::Command;

use color_eyre::Result;
use eyre::{bail, eyre};
use serde::{de::DeserializeOwned, Deserialize};
use tracing::{debug, trace};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct CallerIdentity {
    pub user_id: String,
    pub arn: String,
    pub account: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Credentials {
    pub session_token: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct AssumedRoleUser {
    pub assumed_role_id: String,
    pub arn: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct AssumedRole {
    pub credentials: Credentials,
    pub assumed_role_user: AssumedRoleUser,
}

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

#[derive(Clone, Debug)]
pub struct Aws {
    profile: Option<String>,
    credentials: Option<Credentials>,
}

impl Aws {
    pub fn new(profile: Option<String>) -> Self {
        Self {
            profile,
            credentials: None,
        }
    }

    pub fn with_creds(&self, cred: Credentials) -> Self {
        Self {
            credentials: Some(cred),
            ..self.clone()
        }
    }

    fn run_cmd<T>(&self, args: &[&str]) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let output = self.run_cmd_raw(args)?;
        Ok(if let Some(output) = output {
            Some(serde_json::from_slice(&output)?)
        } else {
            None
        })
    }

    fn run_cmd_raw(&self, args: &[&str]) -> Result<Option<Vec<u8>>> {
        let output = {
            let mut cmd = Command::new("aws");
            let mut cmd = &mut cmd;

            if let Some(creds) = self.credentials.as_ref() {
                cmd = cmd
                    .env("AWS_SESSION_TOKEN", &creds.session_token)
                    .env("AWS_ACCESS_KEY_ID", &creds.access_key_id)
                    .env("AWS_SECRET_ACCESS_KEY", &creds.secret_access_key);
            }

            if let Some(profile) = self.profile.as_ref() {
                cmd = cmd.arg("--profile").arg(profile);
            }

            let cmd = cmd.args(args);

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

    pub fn create_or_get_organization(&self) -> Result<Organization> {
        Ok(
            match self.run_cmd::<OrganizationDetails>(&[
                "organizations",
                "create-organization",
                "--feature-set",
                "ALL",
            ])? {
                Some(org) => org.organization,
                None => {
                    self.run_cmd::<OrganizationDetails>(&[
                        "organizations",
                        "describe-organization",
                    ])?
                    .ok_or_else(|| eyre!("Failed to create/fetch existing organization details"))?
                    .organization
                }
            },
        )
    }

    pub(crate) fn list_existing_accouns(&self) -> Result<Vec<Account>> {
        Ok(self
            .run_cmd::<AccountList>(&["organizations", "list-accounts"])?
            .ok_or_else(|| eyre!("Could not get list of accounts"))?
            .accounts)
    }

    pub(crate) fn create_account(&self, account_name: &str, email: &str) -> Result<String> {
        Ok(String::from_utf8(
            self.run_cmd_raw(&[
                "organizations",
                "create-account",
                "--email",
                &email,
                "--account-name",
                account_name,
            ])?
            .ok_or_else(|| eyre!("Failed to create account: {account_name}"))?,
        )?)
    }

    pub(crate) fn get_caller_identity(&self) -> Result<CallerIdentity> {
        Ok(self
            .run_cmd(&["sts", "get-caller-identity"])?
            .ok_or_else(|| eyre!("Failed to check account identity"))?)
    }

    pub(crate) fn assume_account_root_role(&self, account: &Account) -> Result<AssumedRole> {
        Ok(self
            .run_cmd::<AssumedRole>(&[
                "sts",
                "assume-role",
                "--role-session-name",
                &account.name,
                "--role-arn",
                &format!(
                    "arn:aws:iam::{}:role/OrganizationAccountAccessRole",
                    account.id
                ),
            ])?
            .ok_or_else(|| eyre!("Could not assume root role for {}", account.name))?)
    }

    pub fn deploy_cf(&self, stack_name: &str, path: &std::path::Path) -> Result<()> {
        self.run_cmd_raw(&[
            "cloudformation",
            "deploy",
            "--template-file",
            // &String::from_utf8(path.as_os_str().as_bytes())?,
            path.to_str()
                .ok_or_else(|| eyre!("Incorrect path: {}", path.display()))?,
            "--stack-name",
            stack_name,
            "--capabilities",
            "CAPABILITY_NAMED_IAM",
        ])?
        .ok_or_else(|| eyre!("Could nod deploy cloudformation stack {stack_name}"))?;

        Ok(())
    }
}
