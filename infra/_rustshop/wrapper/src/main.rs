use std::{
    ffi::{OsStr, OsString},
    os::unix::prelude::CommandExt,
    path::PathBuf,
};

use color_eyre::Result;
use eyre::{bail, eyre};
use tracing::{debug, info, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "rustshop_env=info,rustshop_terraform=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    color_eyre::install()?;

    let mut args = std::env::args_os();

    trace!(argv = ?args);

    let cmd_base = if let Some(argv0) = args.next() {
        trace!(argv0 = ?argv0);
        let argv0_path = PathBuf::from(argv0.to_owned());
        let cmd_base = argv0_path
            .file_name()
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre!("Invalid argv0: {:?}", &argv0))?;
        cmd_base
    } else {
        bail!("Must have argv0 to detect command being wrapped.");
    };

    let mut cmd = if let Some(argv1) = args.next() {
        trace!(argv1 = ?argv1);
        std::process::Command::new(argv1)
    } else {
        bail!("First argument must be the binary being wrapped");
    };

    let mut cmd = cmd.args(args);

    // No wrapping requested, just exec and don't mess with anything
    if let Some("true") = std::env::var(rustshop_env::Env::NO_WRAP).ok().as_deref() {
        debug!(
            var = rustshop_env::Env::NO_WRAP,
            val = "true",
            "Not modifing the environment - due to env var flag"
        );
        return Err(cmd.exec())?;
    }

    let env = if should_skip_validation(&cmd_base) {
        debug!("Skip aws profile validation");
        rustshop_env::Env::new_detect_no_profile_validation()?
    } else {
        rustshop_env::Env::new_detect()?
    };

    if is_terraform_init() {
        info!("Executing with `terraform init` workaround");

        cmd = cmd.args(&[
            "-backend-config",
            &format!("bucket={}-bootstrap-terraform-state", env.full_account_name),
            "-backend-config",
            &format!("key=state/{}.tfstate", env.account_suffix),
            "-backend-config",
            &format!(
                "dynamodb_table={}-bootstrap-terraform",
                env.full_account_name
            ),
        ]);

        cmd = cmd.args(&[
            "-backend-config",
            &format!("profile={}", env.account.aws_profile),
        ]);
        if let Some(aws_region) = env.account.aws_region.as_ref() {
            cmd = cmd.args(&["-backend-config", &format!("region={}", aws_region)]);
        }
    }

    trace!("Setting `aws` cli envs");
    cmd = env.account.set_aws_envs_on(cmd);
    if cmd_base.to_str() == Some("terraform") {
        trace!("Setting `terraform` envs");
        cmd = env.set_tf_aws_envs_on(cmd)?;
    }

    if cmd_base.to_str() == Some("kops") {
        trace!("Setting `kops` envs");
        cmd = env.set_kops_envs_on(cmd)?;
    }
    trace!(cmd = format!("{cmd:?}"), "exec");
    Err(cmd.exec())?;

    Ok(())
}

fn should_skip_validation(cmd_base: &OsStr) -> bool {
    // If it's `aws configure`, we don't want to mess with it, and it doesn't
    // really
    // This helpwith setting different
    // profiles
    if cmd_base.to_str() != Some("aws") {
        return false;
    }

    let non_flag_args = std::env::args_os()
        .skip(2)
        .filter(|arg| !arg.to_str().map(|s| s.starts_with('-')).unwrap_or(false))
        .take(2)
        .collect::<Vec<_>>();

    trace!(flags = ?non_flag_args, "Checking non-flag `aws` flags");
    match &non_flag_args[..] {
        [cmd, ..] => cmd == &OsString::from("configure"),
        [] => true,
    }
}

fn is_terraform_init() -> bool {
    std::env::args_os()
        .skip(2)
        .filter(|arg| {
            !arg.to_str()
                .map(|arg| arg.starts_with('-'))
                .unwrap_or(false)
        })
        .next()
        .and_then(|arg| arg.to_str().map(ToString::to_string))
        == Some("init".to_string())
}
