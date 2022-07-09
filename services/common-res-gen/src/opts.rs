use clap::{Args, FromArgMatches, Parser};

#[derive(Parser, Debug, Clone)]
// help_template to basically disable `resource-common` showing up as the name
// of the project
#[clap(infer_subcommands = true, help_template = "{usage}\n\n{all-args}")]
pub struct Opts<GeneratorOpts>
where
    GeneratorOpts: FromArgMatches + Args,
{
    #[clap(
        long = "listen",
        short = 'l',
        default_value = "3000",
        env = "LISTEN_PORT"
    )]
    pub listen_port: u16,

    #[clap(flatten)]
    pub app_opts: GeneratorOpts,
}

impl<GeneratorOpts> Opts<GeneratorOpts>
where
    GeneratorOpts: FromArgMatches + Args,
{
    pub fn from_args() -> Self {
        Opts::parse()
    }
}
