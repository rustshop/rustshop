use std::os::unix::prelude::CommandExt;

use color_eyre::Result;
use eyre::bail;
use tracing::{info, trace};
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
    let env = rustshop_env::Env::new_detect()?;

    let use_init_workaround = std::env::args_os()
        .skip(2)
        .filter(|arg| {
            !arg.to_str()
                .map(|arg| arg.starts_with('-'))
                .unwrap_or(false)
        })
        .next()
        .and_then(|arg| arg.to_str().map(ToString::to_string))
        == Some("init".to_string());

    args.next();

    let mut cmd = if let Some(cmd) = args.next() {
        std::process::Command::new(cmd)
    } else {
        bail!("First argument must be the binary being wrapped");
    };
    let mut cmd = cmd.args(args);

    if use_init_workaround {
        info!("Executing `terraform init` workaround");

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

    cmd = env.account.set_aws_envs_on(cmd);
    cmd = env.set_tf_aws_envs_on(cmd)?;

    trace!(cmd = format!("{cmd:?}"), "exec");
    Err(cmd.exec())?;

    Ok(())
}
