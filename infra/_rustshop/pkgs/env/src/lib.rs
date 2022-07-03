use std::{
    io,
    process::{Command, Stdio},
};

use thiserror::Error;
use tracing::{debug, info, trace};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not load env var: {name}")]
    VariableMissing { name: String },

    #[error("Empty env var: {name}. Must populated or unset.")]
    VariableEmpty { name: String },

    #[error("Unable to detect account to use")]
    AccountDirError {
        #[from]
        source: CwdError,
    },

    #[error("Account ID not set")]
    AccountIdNotSet,

    #[error("Profile does not exists")]
    ProfileDoesNotExist,

    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum CwdError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Not under an account directory (child directory of RUSTSHOP_ROOT)")]
    CwdRoot,
    #[error("Cwd is not unicode")]
    CwdUnicode,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
/// Per account envs
pub struct AccountEnvs {
    pub aws_region: Option<String>,
    pub aws_profile: String,
    pub aws_account_id: String,
}

impl AccountEnvs {
    fn load(account_suffix: &str) -> Result<Self> {
        let account_suffix = account_suffix.to_uppercase();
        Ok(Self {
            aws_region: load_env_var_opt(&format!("RUSTSHOP_{}_AWS_REGION", account_suffix))?,
            aws_profile: load_env_var(&format!("RUSTSHOP_{}_AWS_PROFILE", account_suffix))?,
            aws_account_id: load_env_var(&format!("RUSTSHOP_{}_AWS_ACCOUNT_ID", account_suffix))?,
        })
    }

    /// Set the variables that `aws` CLI command expects (and other binaries too)
    pub fn set_aws_envs_on<'cmd>(&self, mut cmd: &'cmd mut Command) -> &'cmd mut Command {
        debug!(AWS_PROFILE = self.aws_profile, "Setting");
        cmd = cmd.env("AWS_PROFILE", &self.aws_profile);

        if let Some(aws_region) = self.aws_region.as_ref() {
            debug!(AWS_REGION = aws_region, "Setting");
            cmd = cmd.env("AWS_REGION", aws_region);
        }

        debug!(AWS_ACCOUNT_ID = self.aws_account_id, "Setting");
        cmd = cmd.env("_AWS_ACCOUNT_ID", &self.aws_account_id);

        cmd
    }
}

#[derive(Debug)]
pub struct Env {
    pub shop_name: String,
    pub account_suffix: String,
    pub full_account_name: String,
    pub account: AccountEnvs,
}

impl Env {
    pub const NO_WRAP: &'static str = "RUSTSHOP_NO_BIN_WRAP";

    pub fn new_detect_no_profile_validation() -> Result<Self> {
        let shop_name = load_env_var("RUSTSHOP_NAME")?;
        let account_suffix = Self::load_account_suffix()?;

        let default_region = load_env_var_opt("RUSTSHOP_DEFAULT_AWS_REGION")?;

        let mut account = AccountEnvs::load(&account_suffix)?;

        account.aws_region = account.aws_region.or(default_region);

        let env = Env {
            full_account_name: format!("{shop_name}-{account_suffix}"),
            shop_name,
            account_suffix,
            account,
        };
        trace!(account = format!("{env:?}"), "Env state");
        info!(
            account = env.account_suffix,
            aws_profile = env.account.aws_profile,
            "Env loaded"
        );

        Ok(env)
    }

    pub fn new_detect() -> Result<Self> {
        let env = Self::new_detect_no_profile_validation()?;
        env.check_profile_exists()?;
        Ok(env)
    }

    fn load_account_suffix() -> Result<String> {
        let root_path = load_env_var("RUSTSHOP_ROOT")?;
        let cwd = std::env::current_dir().map_err(CwdError::Io)?;

        let rel_path = cwd.strip_prefix(root_path).map_err(|_| CwdError::CwdRoot)?;

        Ok(rel_path
            .iter()
            .next()
            .ok_or_else(|| CwdError::CwdRoot)?
            .to_str()
            .ok_or_else(|| CwdError::CwdUnicode)?
            .to_owned())
    }

    /// Set the variables like for `aws` CLI, but prefixed with `TF_VAR_` so they
    /// are visible as Terraform variables.
    pub fn set_tf_aws_envs_on<'cmd>(&self, cmd: &'cmd mut Command) -> Result<&'cmd mut Command> {
        debug!(TF_VAR_SHOPNAME = self.shop_name, "Setting");
        cmd.env("TF_VAR_SHOPNAME", &self.shop_name);

        debug!(TF_VAR_ACCOUNT_SUFFIX = self.account_suffix, "Setting");
        cmd.env("TF_VAR_ACCOUNT_SUFFIX", &self.account_suffix);

        debug!(
            TF_VAR_AWS_ACCOUNT_ID = self.account.aws_account_id,
            "Setting"
        );
        cmd.env("TF_VAR_AWS_ACCOUNT_ID", &self.account.aws_account_id);

        debug!(TF_VAR_AWS_PROFILE = self.account.aws_profile, "Setting");
        cmd.env("TF_VAR_AWS_PROFILE", &self.account.aws_profile);

        if let Some(aws_region) = self.account.aws_region.as_ref() {
            debug!(TF_VAR_AWS_REGION = aws_region, "Setting");
            cmd.env("TF_VAR_AWS_REGION", aws_region);
        }
        Ok(cmd)
    }

    pub(crate) fn check_profile_exists(&self) -> Result<()> {
        let mut cmd = Command::new("aws");
        cmd.env(Env::NO_WRAP, "true");
        cmd.args(&["configure", "get", "name"]);
        self.account.set_aws_envs_on(&mut cmd);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let output = cmd.output()?;

        if output.status.code().unwrap_or(-1) == 255 {
            eprintln!("AWS Profile does not exist!");
            eprintln!("Consider creating it with:");
            eprintln!(
                "aws configure --profile {} set source_profile {}-root",
                self.account.aws_profile, self.shop_name
            );
            eprintln!(
                "aws configure --profile {} set role_arn 'arn:aws:iam::{}:role/OrganizationAccountAccessRole'",
                self.account.aws_profile,
                self.account.aws_account_id,
            );
            return Err(Error::ProfileDoesNotExist);
        }

        Ok(())
    }
}

fn load_env_var_opt(name: &str) -> Result<Option<String>> {
    if let Some(val) = std::env::var(name).ok() {
        if val.is_empty() {
            return Err(Error::VariableEmpty {
                name: name.to_owned(),
            });
        } else {
            Ok(Some(val))
        }
    } else {
        Ok(None)
    }
}

fn load_env_var(name: &str) -> Result<String> {
    let value = std::env::var(name).map_err(|_| Error::VariableMissing {
        name: name.to_owned(),
    })?;

    if value.is_empty() {
        return Err(Error::VariableEmpty {
            name: name.to_owned(),
        });
    }

    Ok(value)
}
