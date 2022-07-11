use clap::{Args, FromArgMatches, Parser};

#[derive(Parser, Debug, Clone)]
// help_template to basically disable `resource-common` showing up as the name
// of the project
#[clap(infer_subcommands = true, help_template = "{usage}\n\n{all-args}")]
pub struct Opts<GeneratorOpts>
where
    GeneratorOpts: FromArgMatches + Args,
{
    #[clap(flatten)]
    pub common: CommonOpts,

    #[clap(flatten)]
    pub custom: GeneratorOpts,
}

#[derive(Args, Debug, Clone)]
pub struct CommonOpts {
    test: bool,
}

impl<GeneratorOpts> Opts<GeneratorOpts>
where
    GeneratorOpts: FromArgMatches + Args,
{
    pub fn from_args() -> Self {
        Opts::parse()
    }
}
