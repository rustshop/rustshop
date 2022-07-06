use std::{ffi::OsString, io};

use clap::{Command, CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[clap(ignore_errors = true, disable_help_flag = true)]
pub struct Completions {
    /// Print out completions script for a given shell
    #[clap(long = "completions")]
    pub completions: Option<clap_complete::Shell>,
}

impl Completions {
    pub fn handle_complections_and_maybe_exit() {
        let opts = Completions::parse();

        if let Some(shell) = opts.completions {
            clap_complete::generate(shell, &mut Opts::command(), "rustshop", &mut io::stdout());
            std::process::exit(0);
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "rustshop",
    about = "Rustshop binary",
    after_help = r#"
Help and feedback: https://github.com/rustshop/rustshop/discussions/categories/help-general
"#
)]

pub struct Opts {
    #[clap(subcommand)]
    pub command: Commands,
}

impl Opts {
    pub fn from_args() -> Opts {
        Opts::parse()
    }

    pub fn command<'help>() -> Command<'help> {
        <Self as CommandFactory>::command()
    }
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    /// Manually add rustshop components to track (see `bootstrap` instead)
    ///
    /// If you are setting up a new shop, use `bootstrap` instead.
    #[clap(subcommand, alias = "a")]
    Add(AddCommands),

    #[clap(subcommand, alias = "a")]
    Bootstrap(BootstrapCommands),

    /// Configure user settings
    #[clap(subcommand, alias = "c")]
    Configure(ConfigureCommands),

    /// Switch current context (account, cluster, namespace)
    #[clap(subcommand, alias = "s")]
    Switch(SwitchCommands),

    #[clap(subcommand, alias = "s")]
    Get(GetCommands),

    /// Wrap a bin supplying rustshop specific arguments and envirionment
    #[clap(hide = true, disable_help_flag = true)]
    #[clap(allow_hyphen_values = true)]
    Wrap {
        #[clap(allow_hyphen_values = true)]
        bin: OsString,

        #[clap(allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
}

#[derive(Debug, Subcommand, Clone)]
pub enum AddCommands {
    Shop {
        /// Shop name. Eg. `rustshop.org`
        #[clap(long = "name", env = "RUSTSHOP_NAME")]
        name: String,

        /// Base DNS domain to use for this shop. Eg. `rustshop.org`
        #[clap(long = "domain", env = "RUSTSHOP_DOMAIN")]
        domain: String,

        /// AWS Account ID
        #[clap(long = "account-id", env = "AWS_ACCOUNT_ID")]
        account_id: String,

        /// AWS Region to bootstrap resources to
        #[clap(long = "region", env = "AWS_REGION", default_value = "us-east-1")]
        aws_region: String,
    },
    Account {
        #[clap(long = "name")]
        name: String,

        /// AWS Account ID
        #[clap(long = "account-id", env = "AWS_ACCOUNT_ID")]
        account_id: String,

        /// AWS Region to bootstrap resources to
        #[clap(long = "region", env = "AWS_REGION", default_value = "us-east-1")]
        aws_region: String,
    },
    Cluster {
        #[clap(long = "name")]
        name: String,
    },
}

#[derive(Debug, Subcommand, Clone)]
pub enum BootstrapCommands {
    Shop {
        /// Shop name. Eg. `rustshop.org`
        ///
        ///  Must be somewhat unique, or there's a risk of bucket name collions.
        #[clap(long = "name", env = "RUSTSHOP_NAME")]
        name: String,

        /// Base DNS domain to use for this shop. Eg. `rustshop.org`
        #[clap(long = "domain", env = "RUSTSHOP_DOMAIN")]
        domain: String,

        // /// AWS Account ID
        // #[clap(long = "account-id", env = "AWS_ACCOUNT_ID")]
        // account_id: String,
        /// AWS Region to bootstrap resources to
        #[clap(long = "region", env = "AWS_REGION", default_value = "us-east-1")]
        aws_region: String,

        /// AWS Profile to use with this account (typically from `~/.aws/config`)
        #[clap(long = "profile", env = "AWS_PROFILE")]
        profile: Option<String>,

        #[clap(flatten)]
        email_opts: EmailBootstrapOpts,
    },
    Account {
        #[clap(long = "name")]
        name: String,

        /// AWS Region to bootstrap resources to
        #[clap(long = "region", env = "AWS_REGION", default_value = "us-east-1")]
        aws_region: String,

        /// AWS Profile to use with this account (typically from `~/.aws/config`)
        #[clap(long = "profile", env = "AWS_PROFILE")]
        profile: Option<String>,

        #[clap(flatten)]
        email_opts: EmailBootstrapOpts,
    },
}

#[derive(Parser, Debug, Clone)]
pub struct EmailBootstrapOpts {
    /// Base account email to use (<user>@<domain>)
    ///
    /// Accounts' emails will be in the form `<user>+<label-prefix><account><label-suffix>@<domain>`
    #[clap(long = "email")]
    pub email: String,

    /// Email label prefix
    ///
    /// See `email` for more info
    #[clap(long = "email-label-prefix", default_value = "")]
    pub email_label_prefix: String,

    /// Email label suffix
    ///
    /// See `email` for more info
    #[clap(long = "email-label-suffix", default_value = "")]
    pub email_label_suffix: String,
}

#[derive(Debug, Subcommand, Clone)]
pub enum ConfigureCommands {
    Account {
        /// Account name
        #[clap(long = "name")]
        name: String,

        /// AWS Profile to use with this account (typically from `~/.aws/config`)
        #[clap(long = "profile", env = "AWS_PROFILE")]
        profile: String,
    },
    Cluster {
        /// Cluster name
        #[clap(long = "name")]
        name: String,

        /// Kube ctx to use with this cluster (typically from `~/.kube/config`)
        #[clap(long = "ctx", env = "KUBE_CTX")]
        ctx: String,
    },
}
#[derive(Debug, Subcommand, Clone)]
pub enum SwitchCommands {
    #[clap(alias = "ac")]
    Account { name: String },
    #[clap(alias = "cl")]
    Cluster { name: String },
    #[clap(alias = "ns")]
    Namespace { name: String },
}

#[derive(Debug, Subcommand, Clone)]
pub enum GetCommands {
    #[clap(alias = "c")]
    Context,
}
