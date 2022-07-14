#![feature(option_get_or_insert_default)]

use clap::Args;
use common_res_gen::{GenContext, GenResult};

#[derive(Args, Debug, Clone)]
struct Opts {
    #[clap(long, env = "STARTER_IMAGE")]
    image: String,
    #[clap(long = "node-port")]
    node_port: Option<u16>,
}

struct Gen;

impl common_res_gen::Generator for Gen {
    type Opts = Opts;

    fn generate(&mut self, ctx: &mut GenContext, opts: &Self::Opts) -> GenResult<()> {
        let app_name = "starter";

        let selector = ctx.add_standard_deployment(app_name, &opts.image, |_| {});
        if let Some(node_port) = opts.node_port {
            ctx.add_node_port_service(app_name, node_port, &selector, |_| {});
        } else {
            ctx.add_standard_service(app_name, &selector, |_| {});
        }

        Ok(())
    }
}

fn main() -> GenResult<()> {
    common_res_gen::run_resource_generator(Gen)
}
