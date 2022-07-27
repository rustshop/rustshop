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
"#,
    infer_subcommands = true
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
    #[clap(subcommand)]
    Add(AddCommands),

    /// Set up new amazon account/organization
    #[clap(subcommand)]
    Bootstrap(BootstrapCommands),

    /// Configure user settings
    #[clap(subcommand)]
    Configure(ConfigureCommands),

    /// Switch current context (account, cluster, namespace)
    #[clap(subcommand)]
    Switch(SwitchCommands),

    /// Display certain values
    #[clap(subcommand)]
    Get(GetCommands),

    /// Wrap a bin supplying rustshop specific arguments and envirionment
    #[clap(hide = true, disable_help_flag = true)]
    #[clap(allow_hyphen_values = true)]
    Wrap { bin: OsString, args: Vec<OsString> },
}

#[derive(Debug, Subcommand, Clone)]
pub enum AddCommands {
    Shop {
        /// Shop name. Eg. `rustshop.org`
        #[clap(env = "RUSTSHOP_NAME")]
        name: String,

        /// Base DNS domain to use for this shop. Eg. `rustshop.org`
        #[clap(long = "domain", env = "RUSTSHOP_DOMAIN")]
        domain: String,

        /// AWS Region to bootstrap resources to
        #[clap(long = "region", env = "AWS_REGION", default_value = "us-east-1")]
        aws_region: String,
    },
    Account {
        name: String,

        /// AWS Region to bootstrap resources to
        #[clap(long = "region", env = "AWS_REGION", default_value = "us-east-1")]
        aws_region: String,
    },
    Cluster {
        name: String,

        #[clap(name = "account")]
        account: Option<String>,
    },
}

#[derive(Debug, Subcommand, Clone)]
pub enum BootstrapCommands {
    Shop {
        /// Shop name. Eg. `rustshop.org`
        ///
        ///  Must be somewhat unique, or there's a risk of bucket name collions.
        #[clap(env = "RUSTSHOP_NAME")]
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
    Cluster {
        /// Cluster name. Eg. `prod`. Default to current account name.
        #[clap(long = "name")]
        name: Option<String>,

        #[clap(long = "account")]
        account: Option<String>,

        /// Set to true *only* after cluster DNS is set up and working
        #[clap(long = "dns-ready")]
        dns_ready: bool,

        /// Set to true *only* after cluster DNS is set up and working
        #[clap(long = "create-hosted-zone")]
        create_hosted_zone: bool,

        /// Bootstrap minimal, cheapest working cluster possible (1 node + 1 worker, smallest EBSs, spot instances)
        #[clap(long = "minimal")]
        minimal: bool,

        #[clap(allow_hyphen_values = true)]
        other_args: Vec<OsString>,
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
        name: String,

        /// AWS Profile to use with this account (typically from `~/.aws/config`)
        #[clap(long = "profile", env = "AWS_PROFILE")]
        profile: String,
    },
    Cluster {
        /// Cluster name
        name: String,

        #[clap(name = "account")]
        account: Option<String>,

        /// Kube ctx to use with this cluster (typically from `~/.kube/config`)
        #[clap(long = "ctx", env = "KUBE_CTX")]
        ctx: String,
    },
}
#[derive(Debug, Subcommand, Clone)]
pub enum SwitchCommands {
    Account {
        name: String,
    },
    Cluster {
        name: String,
    },
    #[clap(alias = "n", alias = "ns")]
    Namespace {
        name: String,
    },
}

#[derive(Debug, Subcommand, Clone)]
pub enum GetCommands {
    #[clap(alias = "c")]
    Context,
    Account {
        #[clap(long = "profile")]
        profile: bool,
    },
    Cluster,
    Namespace,
}
