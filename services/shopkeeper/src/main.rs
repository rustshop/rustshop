use std::sync::Arc;

use axum::{
    body::{self},
    http::{HeaderMap, HeaderValue},
    response::Html,
    routing::{get, post},
    Extension,
};
use clap::Args;
use common_app::{
    AppResult, RequestError, RequestResult, WebhookVerificationError, WebhookVerificationResult,
};

#[derive(Args, Debug, Clone)]
pub struct Opts {
    #[clap(long = "github-username", env = "SHOPKEEPER_GITHUB_USERNAME")]
    username: String,

    #[clap(long = "github-access-token", env = "SHOPKEEPER_GITHUB_ACCESS_TOKEN")]
    access_token: String,

    #[clap(
        long = "github-webhook-secret",
        env = "SHOPKEEPER_GITHUB_WEBHOOK_SECRET"
    )]
    webhook_secret: String,

    #[clap(long = "github-webhook-url", env = "SHOPKEEPER_GITHUB_WEBHOOK_URL")]
    webhook_url: String,
}

pub struct State {
    opts: Opts,
}

impl State {
    pub fn new(opts: Opts) -> Self {
        Self { opts }
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let app = common_app::AppBuilder::new();
    let (app, opts) = app.parse_opts::<Opts>();

    let shared_state = Arc::new(State::new(opts));

    app.run_axum(|router| {
        Ok(router
            .route("/", get(handler))
            .route("/github/webhook", post(github_webhook_handler))
            .layer(Extension(shared_state)))
    })
    .await?;

    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn github_webhook_handler(
    Extension(state): Extension<Arc<State>>,
    headers: HeaderMap,
    body: body::Bytes,
) -> RequestResult<Html<&'static str>> {
    verify_github_request(
        Some(&state.opts.webhook_secret),
        &body,
        headers.get("x-hub-signature-256"),
    )
    .map_err(RequestError::from)?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::from_slice::<serde_json::Value>(&body)
                .map_err(|_| RequestError::InternalError)?
        )
        .map_err(|_| RequestError::InternalError)?
    );
    Ok(Html("<h1>Hello, World!</h1>"))
}

pub fn verify_github_request(
    secret: Option<&str>,
    payload: &[u8],
    tag: Option<&HeaderValue>,
) -> WebhookVerificationResult<()> {
    use ring::hmac;
    if let Some(secret) = secret {
        if let Some(tag) = tag {
            let sha256_eq_prefix = b"sha256=";
            let tag = tag.as_bytes();
            let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
            let tagbytes = hex::decode(&tag[sha256_eq_prefix.len()..]).map_err(|_| {
                WebhookVerificationError::InvalidTag {
                    tag: tag.to_owned(),
                }
            })?;
            if tag.starts_with(sha256_eq_prefix) {
                hmac::verify(&key, payload, tagbytes.as_slice()).map_err(|_| {
                    WebhookVerificationError::InvalidTag {
                        tag: tag.to_owned(),
                    }
                })?;
            } else {
                Err(WebhookVerificationError::InvalidTag {
                    tag: tag.to_owned(),
                })?
            }
        } else {
            Err(WebhookVerificationError::MissingTag)?
        }
    }
    Ok(())
}
