use axum::{response::Html, routing::get};
use clap::Args;
use common_app::AppResult;

struct App;

#[derive(Args, Debug, Clone)]
struct Opts;

impl common_app::App for App {
    type Opts = Opts;

    fn pre_serve(&mut self, router: axum::Router) -> AppResult<axum::Router> {
        Ok(router.route("/", get(handler)))
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    common_app::run_axum_app(App).await
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
