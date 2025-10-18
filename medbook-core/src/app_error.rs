use std::error::Error;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::aliases::DieselError;

#[derive(Serialize, Deserialize)]
pub struct StdResponse<T: Serialize, M: ToString> {
    pub data: Option<T>,
    pub message: Option<M>,
}

impl<T: Serialize, M: ToString + Serialize> IntoResponse for StdResponse<T, M> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Service {0} is unreachable")]
    ServiceUnreachable(String),

    #[error(
        "Resource not found - it is possible that the user is trying to access a forbidden resource"
    )]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Forbidden resource: {0}")]
    ForbiddenResource(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Error: {}", self.to_string());
        error!("Detailed error: {:#?}", self.source());

        let (status, message) = match &self {
            AppError::ServiceUnreachable(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ForbiddenResource(_) => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };

        (
            status,
            Json(StdResponse::<(), String> {
                data: None,
                message: Some(message),
            }),
        )
            .into_response()
    }
}

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> Self {
        match &err {
            DieselError::DatabaseError(kind, _info) => match kind {
                _ => AppError::Other(anyhow::Error::new(err)),
            },
            DieselError::NotFound => AppError::Other(anyhow::Error::new(err)),
            _ => AppError::Other(anyhow::Error::new(err)),
        }
    }
}
