use std::{
    ffi::{OsStr, OsString},
    os::unix::prelude::CommandExt,
    path::PathBuf,
    process::Command,
};

use derive_more::Display;
use error_stack::{Context, IntoReport, Result, ResultExt};
use rustshop_env::{AccountCfg, Env, ShopAccountCfg, ShopClusterCfg};
use tracing::{debug, info, trace};

#[derive(Debug, Display)]
pub enum WrapError {
    #[display(fmt = "Invalid binary: {}", "bin.to_string_lossy()")]
    InvalidBin { bin: OsString },
    #[display(fmt = "Exec failed")]
    ExecFailed,
    #[display(fmt = "Loading rustshop env failed")]
    EnvFailure,
}

impl Context for WrapError {}

pub type WrapResult<T> = Result<T, WrapError>;

pub fn exec_wrapped_bin(bin: OsString, args: Vec<OsString>) -> WrapResult<()> {
    let bin_base_name = PathBuf::from(&bin)
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(|| WrapError::InvalidBin { bin: bin.clone() })?;

    let mut cmd = std::process::Command::new(bin);

    // No wrapping requested, just exec and don't mess with anything
    if let Some("true") = std::env::var(rustshop_env::Env::NO_BIN_WRAP_ENV_NAME)
        .ok()
        .as_deref()
    {
        debug!(
            var = rustshop_env::Env::NO_BIN_WRAP_ENV_NAME,
            val = "true",
            "Not modifing the environment - due to env var flag"
        );
        trace!("Exec: {cmd:?}");
        Err(cmd.args(&args).exec())
            .into_report()
            .change_context(WrapError::ExecFailed)?;
    }

    let env = Env::load().change_context(WrapError::EnvFailure)?;

    let context = env
        .get_context_account()
        .change_context(WrapError::EnvFailure)?;

    let account_cfg = &context
        .account
        .expect("account set checked in get_context_account")
        .1;

    if is_terraform_init(&bin_base_name, &args) {
        info!("Executing with `terraform init` workaround");

        cmd.args(&[
            &format!(
                "-backend-config=bucket={}-bootstrap-terraform-state",
                account_cfg.shop.bootstrap_name
            ),
            &format!("-backend-config=key={}.tfstate", account_cfg.shop.bootstrap_name),
            &format!(
                "-backend-config=dynamodb_table={}-bootstrap-terraform",
                account_cfg.shop.bootstrap_name
            ),
            &format!("-backend-config=profile={}", account_cfg.user.aws_profile),
            &format!("-backend-config=region={}", account_cfg.shop.bootstrap_aws_region),
        ]);
    }

    trace!("Setting `aws` cli envs");
    set_aws_envs_on(account_cfg, &mut cmd);
    if bin_base_name.to_str() == Some("terraform") {
        trace!("Setting `terraform` envs");
        set_tf_aws_envs_on(&env, account_cfg, &mut cmd)?;
    }

    if bin_base_name.to_str() == Some("kops") {
        let context = env
            .get_context_cluster()
            .change_context(WrapError::EnvFailure)?;

        trace!("Setting `kops` envs");
        set_kops_envs_on(
            &account_cfg.shop,
            &context
                .cluster
                .expect("get_context_cluster checked it")
                .1
                .shop,
            &mut cmd,
        )?;
    }

    match bin_base_name.to_str() {
        // helm and kubectl have the similiar CLI behavior and they tolerate multiple `--context` and `--namespace`
        // arguments, with the following ones overrding the previous ones; so we can just add these as defaults
        n @ Some("kubectl") | n @ Some("helm") => {
            let cfg = env.get_context().change_context(WrapError::EnvFailure)?;

            if is_kubectl_switch(&bin_base_name, &args) {
                let mut new_cmd = std::process::Command::new("rustshop");
                new_cmd.args(&args);
                return Err(new_cmd.exec())
                    .into_report()
                    .change_context(WrapError::ExecFailed)?;
            }

            if let Some(cluster) = cfg.cluster {
                trace!("Adding `--context` to `kubectl`");
                cmd.args(&[
                    // well, actually helm named it differently
                    if n == Some("helm") {
                        "--kube-context"
                    } else {
                        "--context"
                    },
                    &cluster.1.user.kube_ctx,
                ]);
                if let Some(namespace) = cfg.namespace {
                    trace!("Adding `--namespace` to `kubectl`");
                    cmd.args(&["--namespace", &namespace]);
                }
            }
        }
        _ => {}
    }

    cmd.args(&args);

    trace!("Exec: {cmd:?}");
    Err(cmd.exec())
        .into_report()
        .change_context(WrapError::ExecFailed)?;

    Ok(())
}

fn is_terraform_init(base_bin: &OsStr, args: &[OsString]) -> bool {
    base_bin.to_str() == Some("terraform")
        && args
            .iter()
            .filter(|arg| {
                !arg.to_str()
                    .map(|arg| arg.starts_with('-'))
                    .unwrap_or(false)
            })
            .next()
            .and_then(|arg| arg.to_str().map(ToString::to_string))
            == Some("init".to_string())
}

fn is_kubectl_switch(base_bin: &OsStr, args: &[OsString]) -> bool {
    base_bin.to_str() == Some("kubectl")
        && args
            .iter()
            .filter(|arg| {
                !arg.to_str()
                    .map(|arg| arg.starts_with('-'))
                    .unwrap_or(false)
            })
            .next()
            .and_then(|arg| arg.to_str().map(ToString::to_string))
            .map(|arg| args.len() <= "switch".len() && arg[..] == "switch"[..arg.len()])
            .unwrap_or(false)
}

/// Set the variables that `aws` CLI command expects (and other binaries too)
pub fn set_aws_envs_on<'cmd>(
    account_cfg: &AccountCfg,
    mut cmd: &'cmd mut Command,
) -> &'cmd mut Command {
    debug!(AWS_PROFILE = account_cfg.user.aws_profile, "Setting");
    cmd = cmd.env("AWS_PROFILE", &account_cfg.user.aws_profile);

    // if let Some(aws_region) = self.aws_region.as_ref() {
    //     debug!(AWS_REGION = aws_region, "Setting");
    //     cmd = cmd.env("AWS_REGION", aws_region);
    // }

    cmd
}

/// Set the variables like for `aws` CLI, but prefixed with `TF_VAR_` so they
/// are visible as Terraform variables.
pub fn set_tf_aws_envs_on<'cmd>(
    env: &Env,
    account_cfg: &AccountCfg,
    cmd: &'cmd mut Command,
) -> WrapResult<&'cmd mut Command> {
    debug!(TF_VAR_SHOPNAME = env.shop_cfg().name, "Setting");
    cmd.env("TF_VAR_SHOPNAME", &env.shop_cfg().name);

    debug!(
        TF_VAR_ACCOUNT_BOOTSTRAP_NAME = account_cfg.shop.bootstrap_name,
        "Setting"
    );
    cmd.env(
        "TF_VAR_ACCOUNT_BOOTSTRAP_NAME",
        &account_cfg.shop.bootstrap_name,
    );

    debug!(
        TF_VAR_ACCOUNT_BOOTSTRAP_AWS_REGION = account_cfg.shop.bootstrap_aws_region,
        "Setting"
    );
    cmd.env(
        "TF_VAR_ACCOUNT_BOOTSTRAP_AWS_REGION",
        &account_cfg.shop.bootstrap_aws_region,
    );

    debug!(
        TF_VAR_AWS_PROFILE = &account_cfg.user.aws_profile,
        "Setting"
    );
    cmd.env("TF_VAR_AWS_PROFILE", &account_cfg.user.aws_profile);

    debug!(
        TF_VAR_AWS_REGION = account_cfg.shop.bootstrap_aws_region,
        "Setting"
    );
    cmd.env("TF_VAR_AWS_REGION", &account_cfg.shop.bootstrap_aws_region);
    Ok(cmd)
}

pub fn set_kops_envs_on<'cmd>(
    account_cfg: &ShopAccountCfg,
    cluster_cfg: &ShopClusterCfg,
    cmd: &'cmd mut Command,
) -> WrapResult<&'cmd mut Command> {
    let kops_state_store = get_kops_state_store_url(account_cfg);
    let kops_cluster_name = &cluster_cfg.domain;

    debug!(KOPS_STATE_STORE = kops_state_store, "Setting");
    cmd.env("KOPS_STATE_STORE", &kops_state_store);

    debug!(KOPS_CLUSTER_NAME = kops_cluster_name, "Setting");
    cmd.env("KOPS_CLUSTER_NAME", &kops_cluster_name);

    Ok(cmd)
}

pub fn get_kops_state_store_url(account_cfg: &ShopAccountCfg) -> String {
    format!("s3://{}-bootstrap-kops-state", account_cfg.bootstrap_name)
}
