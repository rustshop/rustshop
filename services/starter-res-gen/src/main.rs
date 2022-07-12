#![feature(option_get_or_insert_default)]

use clap::Args;
use common_res_gen::{GenContext, GenResult};

#[derive(Args, Debug, Clone)]
struct Opts {
    #[clap(long, env = "STARTER_IMAGE")]
    image: String,
}

struct Gen;

impl common_res_gen::Generator for Gen {
    type Opts = Opts;

    fn generate(&mut self, ctx: &mut GenContext, opts: &Self::Opts) -> GenResult<()> {
        let app_name = "starter";

        let selector = ctx.add_standard_deployment(app_name, &opts.image, |_| {});
        ctx.add_standard_service(app_name, &selector, |_| {});

        Ok(())
    }
}

fn main() -> GenResult<()> {
    common_res_gen::run_resource_generator(Gen)
}
