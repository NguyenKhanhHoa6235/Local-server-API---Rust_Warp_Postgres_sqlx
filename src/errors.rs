use thiserror::Error;
use warp::http::StatusCode;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not Found")]
    NotFound,

    #[error("Internal Server Error: {0}")]
    InternalError(String),

    #[error("User already exists")]
    UserExists,

    #[error("You can only delete your own account")]
    NotAllowed,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UserExists => StatusCode::BAD_REQUEST,
            ApiError::NotAllowed => StatusCode::FORBIDDEN,
        }
    }
}

impl warp::reject::Reject for ApiError {}
