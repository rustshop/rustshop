use clap::{Args, FromArgMatches, Parser};

#[derive(Parser, Debug, Clone)]
// help_template to basically disable `app-common` showing up as the name
// of the project
#[clap(infer_subcommands = true, help_template = "{usage}\n\n{all-args}")]
pub struct Opts<AppOpts>
where
    AppOpts: FromArgMatches + Args,
{
    #[clap(
        long = "listen",
        short = 'l',
        default_value = "3000",
        env = "LISTEN_PORT"
    )]
    pub listen_port: u16,

    #[clap(flatten)]
    pub app_opts: AppOpts,
}

impl<AppOpts> Opts<AppOpts>
where
    AppOpts: FromArgMatches + Args,
{
    pub fn from_args() -> Self {
        Opts::parse()
    }
}
