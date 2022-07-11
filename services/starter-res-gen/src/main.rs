use clap::Args;
use common_res_gen::{GenContext, GenResult};

#[derive(Args, Debug, Clone)]
struct Opts;

struct Gen;

impl common_res_gen::Generator for Gen {
    type Opts = Opts;

    fn generate(&mut self, ctx: &mut GenContext, _opts: &Self::Opts) -> GenResult<()> {
        let app_name = "starter";
        let mut selector = ctx.new_selector();

        selector.insert("app", app_name);

        ctx.service(app_name)
            .label("somelabel", "some")
            .port(
                "http",
                common_app::DEFAULT_LISTEN_PORT,
                common_app::DEFAULT_LISTEN_PORT,
            )
            .selector(&selector)
            .build_service()
            .deployement(app_name)
            .replicas(1)
            .selector_match_labels(&selector);


        ctx.service(
            |s| s
            .labels(labels)
            .port("http", 3333, 333)
            .selector(&selector)
        )
        Ok(())
    }
}

fn main() -> GenResult<()> {
    common_res_gen::run_resource_generator(Gen)
}
