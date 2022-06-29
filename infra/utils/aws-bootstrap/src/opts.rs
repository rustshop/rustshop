use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "aws-bootstrap",
    about = "Bootstrap AWS accounts(s) and terraform state",
    long_about = r#"
Bootstrap an AWS account:

 * Create an organization
 * Create new (sub-) accounts in this organization (like `dev` and `prod`)
 * In each account set up a basic cloudformation stack with tiny and cheap
   starting resources and policies that are best practice for self-hosted
   Infrastrucutre-as-a-Code Terraform-based devops system.

Example (create a rustshop-{dev,prod} accounts using AWS CLI `rustshop` profile):

aws-bootstrap --base rustshop --profile rustshop --email infra@rustshop.org
"#,
    after_help = r#"
Help and feedback: https://github.com/rustshop/rustshop/discussions/categories/help-general
"#
)]
pub struct Opts {
    /// AWS profile to use when calling `aws` CLI
    #[clap(long = "profile", env = "AWS_PROFILE")]
    pub profile: Option<String>,

    /// Base name of the account to bootstrap
    ///
    /// Usually your domain name without the TLD.
    #[clap(long = "base")]
    pub base_account_name: String,

    /// comma-separate list of (sub-)accounts to create and bootstrap
    #[clap(
        long = "accounts",
        multiple = true,
        use_value_delimiter = true,
        default_value = "dev,prod"
    )]
    pub accounts: Vec<String>,

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

pub fn parse() -> Opts {
    Opts::parse()
}
