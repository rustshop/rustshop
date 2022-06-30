use std::os::unix::prelude::CommandExt;

use color_eyre::{eyre, Result};
use eyre::eyre;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "terraform_wrapper=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    color_eyre::install()?;

    let args = std::env::args_os();

    let use_init_workaround = std::env::args_os()
        .skip(1)
        .filter(|arg| {
            !arg.to_str()
                .map(|arg| arg.starts_with('-'))
                .unwrap_or(false)
        })
        .next()
        .and_then(|arg| arg.to_str().map(ToString::to_string))
        == Some("init".to_string());

    let mut cmd = std::process::Command::new("terraform");
    let mut cmd = cmd.args(args.skip(1));

    if use_init_workaround {
        let shop_name = std::env::var("TF_VAR_SHOPNAME")?;
        let account_suffix = std::env::current_dir()?
            .file_name()
            .ok_or_else(|| eyre!("Could not read account_suffix from CWD"))?
            .to_str()
            .ok_or_else(|| eyre!("CWD not valid utf8"))?
            .to_owned();

        let account_name = format!("{shop_name}-{account_suffix}");
        let aws_region = std::env::var("TF_VAR_AWS_REGION")?;

        info!("Account name: {account_name}");

        cmd = cmd.args(&[
            "-backend-config",
            &format!("bucket={account_name}-bootstrap-terraform"),
            "-backend-config",
            &format!("key=state/{account_name}.tfstate"),
            "-backend-config",
            &format!("dynamodb_table={account_name}-bootstrap-terraform"),
            "-backend-config",
            &format!("region={aws_region}"),
            // "-backend-config",
            // &format!("profile={account_name}"),
        ]);
    }
    Err(cmd.exec())?;

    Ok(())
}
