use axum::{response::Html, routing::get};
use common_app::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    let app = common_app::AppBuilder::new();
    let (app, _opts) = app.parse_opts::<common_app::NoOpts>();

    app.run_axum(|router| Ok(router.route("/", get(handler))))
        .await?;

    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
