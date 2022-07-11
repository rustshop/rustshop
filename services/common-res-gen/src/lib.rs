#![feature(option_get_or_insert_default)]

use derive_more::From;
use error_stack::{IntoReport, Result, ResultExt};
use k8s_openapi::api::apps::v1::Deployment;
use std::{
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
    Deployement(k8s::Deployment),
    Service(k8s::Service),
}

// Maybe use https://docs.rs/impl-enum instead?
impl serde::Serialize for Resource {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Resource::Deployement(r) => r.serialize(serializer),
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
}

impl GenContext {
    fn new(opts: opts::CommonOpts) -> GenContext {
        Self {
            opts,
            resources: vec![],
        }
    }

    pub fn service<'ctx>(&'ctx mut self, name: &str) -> ServiceBuilder<'ctx> {
        ServiceBuilder {
            ctx: self,
            service: k8s::Service {
                metadata: k8s::ObjectMeta {
                    name: Some(name.to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
            name: name.to_owned(),
        }
        .label("name", name)
    }

    pub fn deployement<'ctx>(&'ctx mut self, name: &str) -> DeploymentBuilder<'ctx> {
        DeploymentBuilder {
            ctx: self,
            deployment: k8s::Deployment {
                metadata: k8s::ObjectMeta {
                    name: Some(name.to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            },
            name: name.to_owned(),
        }
        .label("name", name)
    }
    pub fn new_selector(&self) -> LabelSet {
        LabelSet::default()
    }
}

pub struct ServiceBuilder<'ctx> {
    ctx: &'ctx mut GenContext,
    name: String,
    service: k8s::Service,
}

impl<'ctx> ServiceBuilder<'ctx> {
    pub fn label(mut self, name: &str, value: &str) -> Self {
        match self
            .service
            .metadata
            .labels
            .get_or_insert_default()
            .entry(name.to_owned())
        {
            Vacant(e) => {
                e.insert(value.to_string());
            }
            Occupied(mut e) => {
                warn!(
                    label = name,
                    old = e.get(),
                    new = value,
                    service = self.name;
                    "Overwritting existing label"
                );
                e.insert(value.to_string());
            }
        };

        self
    }

    pub fn port(mut self, name: &str, port: u16, target_port: u16) -> Self {
        self.service
            .spec
            .get_or_insert_default()
            .ports
            .get_or_insert_default()
            .push(k8s::ServicePort {
                name: Some(name.to_owned()),
                port: i32::from(port),
                target_port: Some(k8s::IntOrString::Int(i32::from(target_port))),

                ..Default::default()
            });

        self
    }

    pub fn selector(mut self, selector: &LabelSet) -> Self {
        selector.copy_into(
            self.service
                .spec
                .get_or_insert_default()
                .selector
                .get_or_insert_default(),
        );

        self
    }
    pub fn build_service(self) -> &'ctx mut GenContext {
        self.ctx.resources.push(self.service.into());
        self.ctx
    }
}

pub struct DeploymentBuilder<'ctx> {
    ctx: &'ctx mut GenContext,
    name: String,
    deployment: k8s::Deployment,
}

impl<'ctx> DeploymentBuilder<'ctx> {
    pub fn replicas(mut self, replicas: i32) -> Self {
        self.deployment.spec.get_or_insert_default().replicas = Some(replicas);
        self
    }

    pub fn label(mut self, name: &str, value: &str) -> Self {
        match self
            .deployment
            .metadata
            .labels
            .get_or_insert_default()
            .entry(name.to_owned())
        {
            Vacant(e) => {
                e.insert(value.to_string());
            }
            Occupied(mut e) => {
                warn!(
                    label = name,
                    old = e.get(),
                    new = value,
                    service = self.name;
                    "Overwritting existing label"
                );
                e.insert(value.to_string());
            }
        };

        self
    }

    pub fn labels(&mut self, label_set: &LabelSet) {
        label_set.copy_into(self.deployment.metadata.labels.get_or_insert_default());
    }

    pub fn selector_match_labels(mut self, selector: &LabelSet) -> Self {
        selector.copy_into(
            self.deployment
                .spec
                .get_or_insert_default()
                .selector
                .match_labels
                .get_or_insert_default(),
        );

        self
    }

    pub fn template(&mut self) -> DeploymentTemplate {
        DeploymentTemplate(&mut self.spec.template)
    }

    pub fn build_deployment(self) -> &'ctx mut GenContext {
        self.ctx.resources.push(self.deployment.into());
        self.ctx
    }
}

#[derive(Default)]
pub struct LabelSet(BTreeMap<String, String>);

impl LabelSet {
    pub fn insert(&mut self, name: &str, value: &str) -> &mut Self {
        match self.0.entry(name.to_owned()) {
            Vacant(e) => {
                e.insert(value.to_string());
            }
            Occupied(mut e) => {
                warn!(
                    label = name,
                    old = e.get(),
                    new = value;
                    "Overwritting existing label in a labelset"
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
                        "Overwritting existing label when copying from a labelset"
                    );
                    e.insert(value.to_string());
                }
            }
        }
    }
}


pub struct DeploymentBuilder<'ctx> {
    ctx: &'ctx mut GenContext,
    name: String,
    deployment: k8s::Deployment,
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
