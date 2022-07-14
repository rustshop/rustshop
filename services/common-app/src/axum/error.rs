use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("webhook verification failed")]
    WebhookVerification(#[from] WebhookVerificationError),
    #[error("internal error")]
    InternalError,
}

pub type RequestResult<T> = std::result::Result<T, RequestError>;

#[derive(Error, Debug)]
pub enum WebhookVerificationError {
    #[error("missing verification tag")]
    MissingTag,
    #[error("invalid verification tag")]
    InvalidTag { tag: Vec<u8> },
}

pub type WebhookVerificationResult<T> = std::result::Result<T, WebhookVerificationError>;

impl IntoResponse for RequestError {
    fn into_response(self) -> axum::response::Response {
        let code = match self {
            RequestError::WebhookVerification(e) => match e {
                WebhookVerificationError::MissingTag => StatusCode::BAD_REQUEST,
                WebhookVerificationError::InvalidTag { tag: _ } => StatusCode::UNAUTHORIZED,
            },
            RequestError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (code, code.canonical_reason().unwrap_or("Unknown Error")).into_response()
    }
}
