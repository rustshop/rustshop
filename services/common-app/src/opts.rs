use clap::{Args, FromArgMatches, Parser};

#[derive(Parser, Debug, Clone)]
// help_template to basically disable `common-app` showing up as the name
// of the project
#[clap(infer_subcommands = true, help_template = "{usage}\n\n{all-args}")]
pub struct Opts<AppOpts>
where
    AppOpts: FromArgMatches + Args,
{
    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(flatten)]
    pub app_opts: AppOpts,
}

#[derive(Args, Debug, Clone)]
pub struct CommonOpts {
    // TODO: something better than
    // https://users.rust-lang.org/t/structopt-with-computed-default-value/57985/2?u=dpc
    // ?
    #[clap(
        long = "listen",
        short = 'l',
        default_value = "3000",
        env = "LISTEN_PORT"
    )]
    pub listen_port: u16,
}

impl<AppOpts> Opts<AppOpts>
where
    AppOpts: FromArgMatches + Args,
{
    pub fn from_args() -> Self {
        Opts::parse()
    }
}
