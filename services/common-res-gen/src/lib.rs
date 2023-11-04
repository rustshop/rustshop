#![feature(option_get_or_insert_default)]

use amplify::{bmap, s};
use derive_more::From;
use error_stack::{IntoReport, Result, ResultExt};
use k8s_openapi::{
    api::{
        apps::v1::DeploymentSpec,
        core::v1::{Container, PodSpec, PodTemplateSpec, ServiceSpec},
    },
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
    DeepMerge,
};
use std::{
    borrow::Borrow,
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap,
    },
    io,
};
use thiserror::Error;
use tracing::log::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::opts::Opts;

mod opts;

pub mod k8s {
    pub use k8s_openapi::api::core::v1::ServicePort;
    pub use k8s_openapi::api::core::v1::ServiceSpec;
    pub use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service};
    pub use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
    pub use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    pub use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
}

/// A resource that we allow to be generated
#[derive(From, Debug, Clone)]
pub enum Resource {
    Deployment(k8s::Deployment),
    Service(k8s::Service),
}

// Maybe use https://docs.rs/impl-enum instead?
impl serde::Serialize for Resource {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Resource::Deployment(r) => r.serialize(serializer),
            Resource::Service(r) => r.serialize(serializer),
        }
    }
}

pub trait Generator {
    type Opts: clap::Args;

    fn generate(&mut self, ctx: &mut GenContext, opts: &Self::Opts) -> GenResult<()>;
}

pub struct GenContext {
    #[allow(unused)]
    opts: opts::CommonOpts,
    resources: Vec<Resource>,
    common_labels: LabelSet,
}

impl GenContext {
    fn new(opts: opts::CommonOpts) -> GenContext {
        Self {
            opts,
            resources: vec![],
            common_labels: LabelSet::new().insert("shop", "rustshop").to_owned(),
        }
    }

    pub fn add_plain_service<'ctx>(
        &'ctx mut self,
        name: &str,
        func: impl FnOnce(&mut k8s::Service),
    ) -> &mut Self {
        let mut service = k8s::Service::default();
        service.metadata.merge_from(ObjectMeta {
            name: Some(name.into()),
            labels: self.common_labels.to_owned().into(),
            ..Default::default()
        });
        service.metadata.labels.merge_from(
            bmap! {
                s!("template") => s!("plain"),
                s!("app") => name.to_owned()
            }
            .into(),
        );

        func(&mut service);

        self.resources.push(service.into());
        self
    }

    pub fn add_standard_service<'ctx>(
        &'ctx mut self,
        name: &str,
        pod_selector: &LabelSet,
        func: impl FnOnce(&mut k8s::Service),
    ) -> &mut Self {
        self.add_plain_service(name, |s| {
            s.metadata.labels.merge_from(pod_selector.to_owned().into());
            s.metadata.labels.merge_from(
                bmap! {
                    s!("template") => s!("standard")
                }
                .into(),
            );
            s.spec.get_or_insert_default().merge_from(ServiceSpec {
                ports: vec![k8s::ServicePort {
                    name: "http".to_owned().into(),
                    port: i32::from(3000),
                    ..Default::default()
                }]
                .into(),
                selector: pod_selector.to_owned().into(),
                ..Default::default()
            });
            func(s);
        })
    }

    pub fn add_node_port_service<'ctx>(
        &'ctx mut self,
        name: &str,
        port: u16,
        pod_selector: &LabelSet,
        func: impl FnOnce(&mut k8s::Service),
    ) -> &mut Self {
        self.add_plain_service(name, |s| {
            s.metadata.labels.merge_from(pod_selector.to_owned().into());
            s.metadata.labels.merge_from(
                bmap! {
                    s!("template") => s!("standard")
                }
                .into(),
            );
            s.spec.get_or_insert_default().merge_from(ServiceSpec {
                ports: vec![k8s::ServicePort {
                    name: "http".to_owned().into(),
                    port: i32::from(3000),
                    node_port: i32::from(port).into(),
                    ..Default::default()
                }]
                .into(),
                selector: pod_selector.to_owned().into(),
                ..Default::default()
            });
            func(s);
        })
    }

    pub fn add_plain_deployment<'ctx>(
        &'ctx mut self,
        name: &str,
        func: impl FnOnce(&mut k8s::Deployment),
    ) -> &mut Self {
        let mut deployment = k8s::Deployment::default();

        deployment.metadata.merge_from(ObjectMeta {
            name: Some(name.into()),
            labels: self.common_labels.to_owned().into(),
            ..Default::default()
        });
        deployment.metadata.labels.merge_from(
            bmap! {
                s!("template") => s!("plain"),
                s!("app") => name.to_owned()
            }
            .into(),
        );

        func(&mut deployment);

        self.resources.push(deployment.into());
        self
    }

    pub fn add_standard_deployment<'ctx>(
        &'ctx mut self,
        name: &str,
        image: &str,
        func: impl FnOnce(&mut k8s::Deployment),
    ) -> LabelSet {
        let pod_selector = self.new_labels().insert("app", name);

        self.add_plain_deployment(name, |d| {
            d.metadata.labels.merge_from(pod_selector.to_owned().into());
            d.metadata.labels.merge_from(
                bmap! {
                    s!("template") => s!("standard")
                }
                .into(),
            );
            d.spec.get_or_insert_default().merge_from(DeploymentSpec {
                replicas: Some(1),
                selector: pod_selector.to_owned().into(),
                template: PodTemplateSpec {
                    metadata: ObjectMeta {
                        labels: pod_selector.to_owned().into(),
                        name: name.to_owned().into(),
                        ..Default::default()
                    }
                    .into(),
                    spec: PodSpec {
                        containers: vec![Container {
                            image: image.to_owned().into(),
                            name: "main".to_owned(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                },
                ..Default::default()
            });
            func(d);
        });

        pod_selector
    }

    pub fn new_labels(&self) -> LabelSet {
        LabelSet::default()
    }
}

#[derive(Default, Clone, Debug)]
pub struct LabelSet(BTreeMap<String, String>);

impl LabelSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(mut self, name: &str, value: &str) -> Self {
        match self.0.entry(name.to_owned()) {
            Vacant(e) => {
                e.insert(value.to_string());
            }
            Occupied(mut e) => {
                warn!(
                    label = name,
                    old = e.get(),
                    new = value;
                    "Overwriting existing label in a labelset"
                );
                e.insert(value.to_string());
            }
        }

        self
    }

    pub fn copy_into(&self, dst: &mut BTreeMap<String, String>) {
        for (name, value) in &self.0 {
            match dst.entry(name.to_owned()) {
                Vacant(e) => {
                    e.insert(value.to_string());
                }
                Occupied(mut e) => {
                    warn!(
                        label = name,
                        old = e.get(),
                        new = value;
                        "Overwriting existing label when copying from a labelset"
                    );
                    e.insert(value.to_string());
                }
            }
        }
    }
}

impl Borrow<BTreeMap<String, String>> for &LabelSet {
    fn borrow(&self) -> &BTreeMap<String, String> {
        &self.0
    }
}

impl From<LabelSet> for Option<k8s::LabelSelector> {
    fn from(set: LabelSet) -> Self {
        Some(k8s::LabelSelector {
            match_labels: Some(set.0),
            ..Default::default()
        })
    }
}

impl From<LabelSet> for k8s::LabelSelector {
    fn from(set: LabelSet) -> Self {
        k8s::LabelSelector {
            match_labels: Some(set.0),
            ..Default::default()
        }
    }
}
impl From<LabelSet> for Option<BTreeMap<String, String>> {
    fn from(set: LabelSet) -> Self {
        Some(set.0)
    }
}

#[derive(Error, Debug)]
#[error("Generator error")]
pub struct GenError;

pub type GenResult<T> = Result<T, GenError>;

pub fn run_resource_generator(mut gen: impl Generator) -> GenResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    let opts = Opts::from_args();

    let mut ctx = GenContext::new(opts.common);

    gen.generate(&mut ctx, &opts.custom)?;

    for resource in ctx.resources {
        println!(
            "{}",
            serde_yaml::to_string(&resource)
                .report()
                .change_context(GenError)?
        );
    }

    Ok(())
}
