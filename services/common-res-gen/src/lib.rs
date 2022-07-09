use derive_more::From;
use error_stack::{IntoReport, Result, ResultExt};
use std::io;
use thiserror::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::opts::Opts;

mod opts;

pub use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service};

/// A resource that we allow to be generated
#[derive(From, Debug, Clone)]
pub enum Resource {
    Deployement(Deployment),
    Service(Service),
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

    fn generate(&mut self, opts: &Opts<Self::Opts>) -> GenResult<Vec<Resource>>;
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

    let resources = gen.generate(&opts)?;

    for resource in resources {
        println!(
            "{}",
            serde_yaml::to_string(&resource)
                .report()
                .change_context(GenError)?
        );
    }

    Ok(())
}
