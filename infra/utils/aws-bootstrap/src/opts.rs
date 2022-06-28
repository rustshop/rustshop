use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "aws-bootstrap",
    about = "Bootstrap AWS accounts(s) and terraform state"
)]
pub struct Opts {
    /// Port to listen on
    #[clap(long = "", short = 'l', default_value = "0")]
    pub listen_port: u16,

    #[clap(long = "yes")]
    pub yes: bool,

    #[clap(long = "profile", env = "AWS_PROFILE", default_value = "default")]
    pub profile: String,

    #[clap(long = "accounts")]
    pub accounts: Vec<String>,

    #[clap(long = "email")]
    pub email: String,

    #[clap(long = "email-label-prefix", default_value = "")]
    pub email_label_prefix: String,

    #[clap(long = "email-label-suffix", default_value = "")]
    pub email_label_suffix: String,
}

pub fn parse() -> Opts {
    Opts::parse()
}
