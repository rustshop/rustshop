use std::{net::SocketAddr, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::{Request, StatusCode},
    BoxError, Router,
};
use error_stack::{IntoReport, ResultExt};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, Span};

mod error;
pub use self::error::*;

use crate::{opts, shutdown_signal, AppBuilder, AppError, AppResult};

impl AppBuilder<opts::CommonOpts> {
    pub async fn run_axum(&self, func: impl FnOnce(Router) -> AppResult<Router>) -> AppResult<()> {
        let router = Router::new();

        let router = func(router)?;
        let router = configure_axum_router(router);

        // run it
        let addr = SocketAddr::from(([0, 0, 0, 0], self.common_opts.listen_port));
        info!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .report()
            .change_context(AppError)?;

        Ok(())
    }
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

pub(crate) fn configure_axum_router(router: axum::Router) -> axum::Router {
    router
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(30)),
        )
        .layer(TraceLayer::new_for_http())
        .layer(
            TraceLayer::new_for_http().on_request(|request: &Request<_>, _span: &Span| {
                debug!("{:?}", request);
            }),
        )
}
