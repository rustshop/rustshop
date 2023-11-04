use derive_more::Display;
use error_stack::{bail, Context, Result, ResultExt};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rustshop_env::Env;

use std::process::Command;

use serde::{de::DeserializeOwned, Deserialize};
use tracing::{debug, trace};

#[derive(Debug, Display)]
pub enum AwsError {
    #[display(
        fmt = "Response deserialization failed for `aws {}`",
        "cmd.join(\" \")"
    )]
    ResponseDeserialization { cmd: Vec<String> },
    #[display(fmt = "IO Error")]
    Io,

    #[display(fmt = "`aws` command failed with:\n{}", stderr)]
    CommandFailed { stderr: String },
    #[display(fmt = "Wrong response")]
    WrongResponse,
    #[display(fmt = "Invalid path")]
    InvalidPath,
}

impl Context for AwsError {}

pub type AwsResult<T> = Result<T, AwsError>;

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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct HostedZone {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct ListHostedZones {
    pub hosted_zones: Vec<HostedZone>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct DelegationSet {
    pub name_servers: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct GetHostedZone {
    pub hosted_zone: HostedZone,
    pub delegation_set: DelegationSet,
}

#[derive(Clone, Debug)]
pub struct Aws {
    profile: Option<String>,
    region: String,
    credentials: Option<Credentials>,
}

impl Aws {
    pub fn new(profile: Option<String>, region: String) -> Self {
        Self {
            profile,
            region,
            credentials: None,
        }
    }

    pub fn with_creds(&self, cred: Credentials) -> Self {
        Self {
            credentials: Some(cred),
            ..self.clone()
        }
    }

    fn run_cmd<T>(&self, args: &[&str], ignore_254: bool) -> AwsResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let output = self.run_cmd_raw(args, ignore_254)?;
        Ok(if let Some(output) = output {
            Some(serde_json::from_slice(&output).change_context(
                AwsError::ResponseDeserialization {
                    cmd: args.iter().map(ToString::to_string).collect(),
                },
            )?)
        } else {
            None
        })
    }

    fn run_cmd_raw(&self, args: &[&str], ignore_254: bool) -> AwsResult<Option<Vec<u8>>> {
        let output = {
            let mut cmd = Command::new("aws");
            // We do NOT want to re-wrap the `aws` command we issue directly here
            cmd.env(Env::NO_BIN_WRAP_ENV_NAME, "true");
            cmd.env("AWS_REGION", &self.region);

            if let Some(creds) = self.credentials.as_ref() {
                cmd.env("AWS_SESSION_TOKEN", &creds.session_token)
                    .env("AWS_ACCESS_KEY_ID", &creds.access_key_id)
                    .env("AWS_SECRET_ACCESS_KEY", &creds.secret_access_key)
                    .env_remove("AWS_PROFILE");
            } else if let Some(profile) = self.profile.as_ref() {
                // we only want to set profile if we are not using session tokens
                cmd.arg("--profile").arg(profile);
            }

            cmd.args(args);

            trace!("Running: {:?}", cmd);
            cmd.output().change_context(AwsError::Io)?
        };

        trace!("Status code: {:?}", output.status.code());
        debug!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        debug!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

        // 254 seems to indicate non-fatal error, like
        if ignore_254 && output.status.code().unwrap_or(-1) == 254 {
            return Ok(None);
        }

        if !output.status.success() {
            bail!(AwsError::CommandFailed {
                stderr: String::from_utf8_lossy(&output.stderr).to_string()
            });
        }

        Ok(Some(output.stdout))
    }

    pub fn create_or_get_organization(&self) -> AwsResult<Organization> {
        Ok(
            match self.run_cmd::<OrganizationDetails>(
                &[
                    "organizations",
                    "create-organization",
                    "--feature-set",
                    "ALL",
                ],
                true,
            )? {
                Some(org) => org.organization,
                None => {
                    self.run_cmd::<OrganizationDetails>(
                        &["organizations", "describe-organization"],
                        true,
                    )?
                    .ok_or(AwsError::WrongResponse)?
                    .organization
                }
            },
        )
    }

    pub(crate) fn list_existing_accouns(&self) -> AwsResult<Vec<Account>> {
        Ok(self
            .run_cmd::<AccountList>(&["organizations", "list-accounts"], false)?
            .ok_or(AwsError::WrongResponse)?
            .accounts)
    }

    pub(crate) fn create_account(&self, account_name: &str, email: &str) -> AwsResult<String> {
        String::from_utf8(
            self.run_cmd_raw(
                &[
                    "organizations",
                    "create-account",
                    "--email",
                    &email,
                    "--account-name",
                    account_name,
                ],
                true,
            )?
            .ok_or(AwsError::WrongResponse)?,
        )
        .change_context(AwsError::WrongResponse)
    }

    pub(crate) fn get_caller_identity(&self) -> AwsResult<CallerIdentity> {
        Ok(self
            .run_cmd(&["sts", "get-caller-identity"], false)?
            .ok_or(AwsError::WrongResponse)?)
    }

    pub(crate) fn assume_account_root_role(&self, account: &Account) -> AwsResult<AssumedRole> {
        Ok(self
            .run_cmd::<AssumedRole>(
                &[
                    "sts",
                    "assume-role",
                    "--role-session-name",
                    &account.name,
                    "--role-arn",
                    &format!(
                        "arn:aws:iam::{}:role/OrganizationAccountAccessRole",
                        account.id
                    ),
                ],
                false,
            )?
            .ok_or(AwsError::WrongResponse)?)
    }

    pub fn deploy_cf(&self, stack_name: &str, path: &std::path::Path) -> AwsResult<()> {
        self.run_cmd_raw(
            &[
                "cloudformation",
                "deploy",
                "--template-file",
                path.to_str().ok_or(AwsError::InvalidPath)?,
                "--stack-name",
                stack_name,
                "--capabilities",
                "CAPABILITY_NAMED_IAM",
            ],
            false,
        )?
        .ok_or(AwsError::WrongResponse)?;
        Ok(())
    }

    pub fn configure_set(&self, name: &str, value: &str) -> AwsResult<()> {
        String::from_utf8(
            self.run_cmd_raw(&["configure", "set", name, value], true)?
                .ok_or(AwsError::WrongResponse)?,
        )
        .change_context(AwsError::WrongResponse)?;
        Ok(())
    }

    pub fn list_hosted_zones(&self) -> AwsResult<Vec<HostedZone>> {
        Ok(self
            .run_cmd::<ListHostedZones>(&["route53", "list-hosted-zones"], false)?
            .ok_or(AwsError::WrongResponse)?
            .hosted_zones)
    }

    pub fn get_hosted_zone(&self, id: &str) -> AwsResult<GetHostedZone> {
        Ok(self
            .run_cmd::<GetHostedZone>(&["route53", "get-hosted-zone", "--id", id], false)?
            .ok_or(AwsError::WrongResponse)?)
    }

    pub(crate) fn create_hosted_zone(
        &self,
        domain: &str,
        caller_id: impl Into<Option<String>>,
    ) -> AwsResult<()> {
        let caller_id = caller_id.into().unwrap_or_else(Self::random_caller_id);

        self.run_cmd_raw(
            &[
                "route53",
                "create-hosted-zone",
                "--name",
                domain,
                "--caller-reference",
                &caller_id,
            ],
            false,
        )?
        .ok_or(AwsError::WrongResponse)?;
        Ok(())
    }

    pub fn random_caller_id() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }
}
