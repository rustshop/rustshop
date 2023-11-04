use std::io;

use derive_more::Display;
use env::{Env, EnvRoot, ShopCfg};
use error_stack::{Context, Result, ResultExt};
use rustshop_env as env;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod aws_api;
mod bootstrap;
mod opts;
mod wrap;
use opts::{AddCommands, BootstrapCommands, Commands, EmailBootstrapOpts, GetCommands, Opts};

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Application error")]
    Other,
    #[display(fmt = "Command failed: {}", stderr)]
    CommandFailed { stderr: String },
}

impl Context for AppError {}

pub type AppResult<T> = Result<T, AppError>;

fn main() -> AppResult<()> {
    let res = main_inner();
    if let Err(ref report) = res {
        for suggestion in report.request_ref::<env::Suggestion>() {
            eprintln!("Suggestion: {}", suggestion);
        }
    }

    res
}

fn main_inner() -> AppResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "rustshop_env=info,rustshop=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    opts::Completions::handle_completions_and_maybe_exit();

    match Opts::from_args().command {
        Commands::Add(cmd) => match cmd {
            AddCommands::Shop {
                name,
                domain,
                aws_region,
            } => {
                let env = EnvRoot::load().change_context(AppError::Other)?;
                env.add_shop(name, domain).change_context(AppError::Other)?;

                let mut env = Env::load().change_context(AppError::Other)?;
                // add a root account right away; notably without cluster
                env.add_account("root", &aws_region)
                    .change_context(AppError::Other)?;
                // add a cluster right away
                env.add_cluster("root", "root")
                    .change_context(AppError::Other)?;
            }
            AddCommands::Account { name, aws_region } => {
                let mut env = Env::load().change_context(AppError::Other)?;
                let _account_cfg = env
                    .add_account(&name, &aws_region)
                    .change_context(AppError::Other)?;
                // add a cluster right away
                env.add_cluster(&name, &name)
                    .change_context(AppError::Other)?;
            }
            AddCommands::Cluster { name, account } => {
                let mut env = Env::load().change_context(AppError::Other)?;

                let account = if let Some(account) = account {
                    account
                } else {
                    let context = env.get_context_account().change_context(AppError::Other)?;
                    context
                        .account
                        .expect("get_account_context took care of it")
                        .0
                };

                env.add_cluster(&account, &name)
                    .change_context(AppError::Other)?;
            }
        },
        Commands::Bootstrap(cmd) => match cmd {
            BootstrapCommands::Shop {
                name,
                domain,
                aws_region,
                email_opts,
                profile,
            } => {
                let profile = profile.unwrap_or_else(|| format!("{}-root", name));

                info!("Using {profile} `aws` profile to bootstrap");

                let env = EnvRoot::load().change_context(AppError::Other)?;

                if let Some(shop_cfg) = env.load_shop_cfg_opt().change_context(AppError::Other)? {
                    if (ShopCfg { domain, name }) != shop_cfg {
                        Err(AppError::Other)
                            .attach_printable_lazy(|| format!("Previous settings: {shop_cfg:?}"))?;
                    }
                } else {
                    env.add_shop(name, domain).change_context(AppError::Other)?;
                }

                bootstrap::bootstrap_org(&profile).change_context(AppError::Other)?;

                bootstrap_account("root", &aws_region, &profile, &email_opts)?;
            }
            BootstrapCommands::Account {
                name,
                aws_region,
                profile,
                email_opts,
            } => {
                let env = Env::load().change_context(AppError::Other)?;
                let profile = profile.unwrap_or_else(|| format!("{}-root", env.shop_cfg().name));

                info!("Using {profile} `aws` profile to bootstrap");
                bootstrap_account(&name, &aws_region, &profile, &email_opts)?;
            }
            BootstrapCommands::Cluster {
                account,
                name,
                dns_ready,
                minimal,
                create_hosted_zone,
                other_args,
            } => {
                bootstrap::bootstrap_cluster(
                    account,
                    name,
                    create_hosted_zone,
                    dns_ready,
                    minimal,
                    &other_args,
                )?;
            }
        },
        Commands::Switch(cmd) => {
            let mut env = Env::load().change_context(AppError::Other)?;
            match cmd {
                opts::SwitchCommands::Account { name } => {
                    env.switch_account(&name).change_context(AppError::Other)?;
                }
                opts::SwitchCommands::Cluster { name } => {
                    env.switch_cluster(&name).change_context(AppError::Other)?;
                }
                opts::SwitchCommands::Namespace { name } => {
                    env.switch_namespace(&name)
                        .change_context(AppError::Other)?;
                }
            }

            let context = env.get_context().change_context(AppError::Other)?;
            env.write_ctx_info_to(context, &mut std::io::stderr())
                .change_context(AppError::Other)?;
        }
        Commands::Configure(cmd) => {
            let mut env = Env::load().change_context(AppError::Other)?;
            match cmd {
                opts::ConfigureCommands::Account { name, profile } => {
                    env.configure_account(&name, &profile)
                        .change_context(AppError::Other)?;
                }
                opts::ConfigureCommands::Cluster { name, account, ctx } => {
                    env.configure_cluster(account.as_deref(), &name, &ctx)
                        .change_context(AppError::Other)?;
                }
            }
        }
        Commands::Get(cmd) => match cmd {
            GetCommands::Context => {
                let env = Env::load().change_context(AppError::Other)?;
                let context = env.get_context().change_context(AppError::Other)?;
                env.write_ctx_info_to(context, &mut std::io::stdout())
                    .change_context(AppError::Other)?;
            }
            GetCommands::Account { profile } => {
                let env = Env::load().change_context(AppError::Other)?;
                let context = env.get_context().change_context(AppError::Other)?;
                if let Some(acc) = context.account {
                    if profile {
                        println!("{}", acc.1.user.aws_profile);
                    } else {
                        println!("{}", acc.0);
                    }
                }
            }
            GetCommands::Cluster => {
                let env = Env::load().change_context(AppError::Other)?;
                let context = env.get_context().change_context(AppError::Other)?;
                if let Some(cluster) = context.cluster {
                    println!("{}", cluster.0);
                }
            }
            GetCommands::Namespace => {
                let env = Env::load().change_context(AppError::Other)?;
                let context = env.get_context().change_context(AppError::Other)?;
                if let Some(ns) = context.namespace {
                    println!("{}", ns);
                }
            }
        },
        Commands::Wrap { bin, args } => {
            wrap::exec_wrapped_bin(bin, args).change_context(AppError::Other)?
        }
    }

    Ok(())
}

fn bootstrap_account(
    name: &str,
    aws_region: &str,
    profile: &str,
    email_opts: &EmailBootstrapOpts,
) -> AppResult<()> {
    let mut env = Env::load().change_context(AppError::Other)?;

    let bootstrap_name = format!("{}-{}", env.get_shop_ref().name, name);

    let create_account = if let Some(existing_account) = env
        .get_account_ref_opt(name)
        .change_context(AppError::Other)?
    {
        if existing_account.shop.bootstrap_aws_region != aws_region {
            Err(AppError::Other).attach_printable_lazy(|| {
                "Region is different from the previously used one".to_string()
            })?;
        }

        if existing_account.shop.bootstrap_name != bootstrap_name {
            Err(AppError::Other).attach_printable_lazy(|| {
                "Account full name is different from the previously used one".to_string()
            })?;
        }
        false
    } else {
        true
    };

    let account_id = bootstrap::bootstrap_account(
        &env.get_shop_ref().name,
        name,
        profile,
        aws_region,
        email_opts,
    )
    .change_context(AppError::Other)?;

    if create_account {
        env.add_account(name, aws_region)
            .change_context(AppError::Other)?;
        env.add_cluster(name, name)
            .change_context(AppError::Other)?;
    }

    if account_id
        != bootstrap::get_root_account_id(Some(profile)).change_context(AppError::Other)?
    {
        let aws = aws_api::Aws::new(
            Some(bootstrap_name.clone()),
            "us-east-1".to_string(), /* doesn't matter */
        );

        aws.configure_set(
            "role_arn",
            &format!(
                "arn:aws:iam::{}:role/OrganizationAccountAccessRole",
                account_id
            ),
        )
        .change_context(AppError::Other)?;
        aws.configure_set("source_profile", profile)
            .change_context(AppError::Other)?;

        env.configure_account(name, &bootstrap_name)
            .change_context(AppError::Other)?;
    } else {
        env.configure_account(name, profile)
            .change_context(AppError::Other)?;
    }

    Ok(())
}
