use clap::Args;
use std::io;

use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use thiserror::Error;

use crate::opts::Opts;

mod opts;

pub mod axum;
pub use self::axum::*;

// make sure it matches `Opts`
pub const DEFAULT_LISTEN_PORT: u16 = 3000;

#[derive(Args, Debug, Clone)]
pub struct NoOpts;

#[derive(Debug)]
pub struct AppBuilder<O> {
    common_opts: O,
}

impl AppBuilder<NoOpts> {
    pub fn new() -> Self {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ))
            .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
            .init();

        AppBuilder {
            common_opts: NoOpts,
        }
    }

    pub fn parse_opts<AppOpts>(self) -> (AppBuilder<opts::CommonOpts>, AppOpts)
    where
        AppOpts: clap::FromArgMatches + clap::Args,
    {
        let opts = Opts::from_args();
        (
            AppBuilder {
                common_opts: opts.common_opts,
            },
            opts.app_opts,
        )
    }

    // NOTE: there's also `run_axum` in `mod axum`
}

impl Default for AppBuilder<NoOpts> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug)]
#[error("Application error")]
pub struct AppError;

pub type AppResult<T> = error_stack::Result<T, AppError>;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
                _ = ctrl_c => {},
                        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
