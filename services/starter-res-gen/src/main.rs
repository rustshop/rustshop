use clap::Args;
use common_res_gen::{GenResult, Resource};

#[derive(Args, Debug, Clone)]
struct Opts;

struct Gen;

impl common_res_gen::Generator for Gen {
    type Opts = Opts;

    fn generate(&mut self, _opts: &common_res_gen::Opts<Self::Opts>) -> GenResult<Vec<Resource>> {
        Ok(vec![])
    }
}

fn main() -> GenResult<()> {
    common_res_gen::run_resource_generator(Gen)
}
