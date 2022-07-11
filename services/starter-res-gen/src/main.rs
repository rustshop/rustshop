#![feature(option_get_or_insert_default)]

use clap::Args;
use common_res_gen::{k8s::ServicePort, GenContext, GenResult};

#[derive(Args, Debug, Clone)]
struct Opts;

struct Gen;

impl common_res_gen::Generator for Gen {
    type Opts = Opts;

    fn generate(&mut self, ctx: &mut GenContext, _opts: &Self::Opts) -> GenResult<()> {
        let app_name = "starter";
        let pod_selector = ctx.new_labels().insert("app", app_name);
        let labels = pod_selector.clone().insert("shop", "rustshop");

        ctx.add_service(app_name, |s| {
            s.metadata_with(|m| {
                m.labels()
                    .insert("some_extra_service_label".into(), "iamaservice".into());
            })
            .spec()
            .ports_with(|ports| {
                ports.push(ServicePort {
                    port: i32::from(common_app::DEFAULT_LISTEN_PORT),
                    ..Default::default()
                })
            })
            .selector_set(pod_selector.clone());
        })
        .add_deployment(app_name, |d| {
            d.metadata_with(|m| {
                m.labels_insert_from(&labels);
                m.labels()
                    .insert("some_extra_label".into(), "some_extra_value".into());
            })
            .spec()
            .replicas_set(1)
            .selector_set(pod_selector.clone())
            .template_with(|t| {
                t.metadata().labels_insert_from(&labels);
                t.metadata().name_set(app_name.to_owned());
                t.spec().containers_push_with(|c| {
                    c.image_set("redis".to_owned()).name_set("main");
                });
            });
        });

        Ok(())
    }
}

fn main() -> GenResult<()> {
    common_res_gen::run_resource_generator(Gen)
}
