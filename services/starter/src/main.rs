use app_common::AppResult;
use axum::{response::Html, routing::get};
use clap::Args;

struct App;

#[derive(Args, Debug, Clone)]
struct Opts;

impl app_common::App for App {
    type Opts = Opts;

    fn pre_serve(&mut self, router: axum::Router) -> AppResult<axum::Router> {
        Ok(router.route("/", get(handler)))
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    app_common::run_axum_app(App).await
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
