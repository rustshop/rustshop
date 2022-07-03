use std::os::unix::prelude::CommandExt;

use color_eyre::Result;
use eyre::bail;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "rustshop_env=info,rustshop_aws=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    color_eyre::install()?;

    let mut args = std::env::args_os();
    args.next();

    let bin = if let Some(cmd) = args.next() {
        cmd
    } else {
        bail!("First argument must be the binary being wrapped");
    };

    let mut cmd = std::process::Command::new(bin);
    let mut cmd = cmd.args(args);

    // If it's `aws configure`, we don't want to mess with it.
    let is_aws_configure = std::env::args_os()
        .skip(2)
        .filter(|arg| {
            !arg.to_str()
                .map(|arg| arg.starts_with('-'))
                .unwrap_or(false)
        })
        .next()
        .and_then(|arg| arg.to_str().map(ToString::to_string))
        == Some("configure".to_string());

    if let Some("true") = std::env::var(rustshop_env::Env::NO_WRAP).ok().as_deref() {
        return Err(cmd.exec())?;
    }

    let env = if is_aws_configure {
        rustshop_env::Env::new_detect_no_profile_validation()?
    } else {
        rustshop_env::Env::new_detect()?
    };

    cmd = env.account.set_aws_envs_on(cmd);

    Err(cmd.exec())?;

    Ok(())
}
