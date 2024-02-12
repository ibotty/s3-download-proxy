use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use foundations::telemetry::log;

pub(crate) enum AppError {
    AnyError(anyhow::Error),
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::AnyError(e) => {
                log::error!("{}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong.")
            }
            Self::Unauthorized => {
                log::warn!("Unauthorized!");
                (StatusCode::UNAUTHORIZED, "The secret is invalid")
            }
        }
        .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::AnyError(err.into())
    }
}
