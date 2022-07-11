use axum::Router;
use std::{io, net::SocketAddr};

use error_stack::{IntoReport, Result, ResultExt};
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use thiserror::Error;

use crate::opts::Opts;

mod opts;

// make sure it matches `Opts`
pub const DEFAULT_LISTEN_PORT: u16 = 3000;

pub trait App {
    type Opts: clap::Args;

    fn handle_args(&mut self, _opts: &Self::Opts) -> AppResult<()> {
        Ok(())
    }

    fn pre_serve(&mut self, router: Router) -> AppResult<Router> {
        Ok(router)
    }
}

#[derive(Error, Debug)]
#[error("Application error")]
pub struct AppError;

pub type AppResult<T> = Result<T, AppError>;

pub async fn run_axum_app(mut app: impl App) -> AppResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    let opts = Opts::from_args();

    app.handle_args(&opts.app_opts)?;

    // build our application with a route
    let router = Router::new();

    let router = app.pre_serve(router)?;

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], opts.listen_port));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .report()
        .change_context(AppError)?;

    Ok(())
}

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
