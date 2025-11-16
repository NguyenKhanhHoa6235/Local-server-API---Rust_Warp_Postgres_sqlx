use thiserror::Error;
use warp::http::StatusCode;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("404 Not Found")]
    NotFound,

    #[error("Internal Server Error")]
    InternalError,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl warp::reject::Reject for ApiError {}
