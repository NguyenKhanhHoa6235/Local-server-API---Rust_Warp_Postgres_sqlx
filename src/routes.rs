use warp::Filter;
use sqlx::PgPool;
use crate::handlers;
use crate::models::{RegisterRequest, LoginRequest};
use crate::jwt;
use crate::errors::ApiError;
use crate::rate_limit::{RateLimiter, with_rate_limit};

pub fn with_auth() -> impl Filter<Extract = (jwt::Claims,), Error = warp::Rejection> + Clone {
    warp::header::<String>("authorization")
        .and_then(|auth_header: String| async move {
            if !auth_header.starts_with("Bearer ") {
                return Err(warp::reject::custom(ApiError::Unauthorized("Missing Bearer token".into())));
            }
            let token = auth_header.trim_start_matches("Bearer ").trim();
            match jwt::verify_token(token).await {
                Ok(claims) => Ok(claims),
                Err(msg) => Err(warp::reject::custom(ApiError::Unauthorized(msg))),
            }
        })
}

pub fn create_routes(pool: PgPool) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let db_filter = warp::any().map(move || pool.clone());

    // Khởi tạo RateLimiter
    let limiter = RateLimiter::new();
    let rate_limit_filter = with_rate_limit(limiter);

    let root = warp::path::end()
        .and(warp::get())
        .and(rate_limit_filter.clone())   // áp dụng rate limit
        .and_then(handlers::root_handler);

    let register = warp::path("register")
        .and(warp::post())
        .and(rate_limit_filter.clone())
        .and(warp::body::json::<RegisterRequest>())
        .and(db_filter.clone())
        .and_then(handlers::register_handler);

    let login = warp::path("login")
        .and(warp::post())
        .and(rate_limit_filter.clone())
        .and(warp::body::json::<LoginRequest>())
        .and(db_filter.clone())
        .and_then(handlers::login_handler);

    let delete = warp::path!("users" / i32)
        .and(warp::delete())
        .and(rate_limit_filter.clone())
        .and(db_filter.clone())
        .and(with_auth())
        .and_then(|id: i32, pool: PgPool, claims: jwt::Claims| async move {
            if claims.sub != id {
                return Err(warp::reject::custom(ApiError::NotAllowed));
            }
            handlers::delete_user_handler(id, pool).await
        });

    root.or(register).or(login).or(delete)
        .recover(|err: warp::Rejection| async move {
            if let Some(e) = err.find::<ApiError>() {
                let code = e.status_code();
                let msg = serde_json::json!({ "error": e.to_string() });
                return Ok(warp::reply::with_status(warp::reply::json(&msg), code));
            }
            let msg = serde_json::json!({ "error": "internal server error" });
            Ok(warp::reply::with_status(warp::reply::json(&msg), warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        })
}

