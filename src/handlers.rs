use crate::models::{RegisterRequest, LoginRequest, UserResponse};
use crate::errors::ApiError;
use crate::db;
use sqlx::PgPool;
use warp::http::StatusCode;

/// Handler cho route root GET "/"
pub async fn root_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "message": "Welcome to Local Server API - Rust!"
    })))
}


/// Handler register
pub async fn register_handler(
    body: RegisterRequest,
    pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    if body.name.trim().is_empty() {
        return Err(warp::reject::custom(ApiError::BadRequest(
            "name cannot be empty".into(),
        )));
    }
    if body.password.len() < 8 {
        return Err(warp::reject::custom(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        )));
    }

    let hash = db::hash_password(&body.password)
        .map_err(|_| warp::reject::custom(ApiError::InternalError))?;

    // Insert user
    let id = db::create_user(&pool, &body.name, &hash)
        .await
        .map_err(|_| {
            warp::reject::custom(ApiError::BadRequest("unable to create user".into()))
        })?;

    // Lấy created_at
    let user_opt = db::get_user_by_name(&pool, &body.name)
        .await
        .map_err(|_| warp::reject::custom(ApiError::InternalError))?;

    let (_id, _name, _hash, created_at) =
        user_opt.ok_or_else(|| warp::reject::custom(ApiError::InternalError))?;

    let resp = UserResponse {
        id,
        name: body.name,
        created_at,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&resp),
        StatusCode::CREATED,
    ))
}

/// Handler login
pub async fn login_handler(
    body: LoginRequest,
    pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    if body.name.trim().is_empty() {
        return Err(warp::reject::custom(ApiError::BadRequest(
            "name cannot be empty".into(),
        )));
    }

    let user_opt = db::get_user_by_name(&pool, &body.name)
        .await
        .map_err(|_| warp::reject::custom(ApiError::InternalError))?;

    let (_id, _name, password_hash, _created_at) = match user_opt {
        Some(u) => u,
        None => return Err(warp::reject::custom(ApiError::Unauthorized)),
    };

    let verified = db::verify_password(&password_hash, &body.password)
        .map_err(|_| warp::reject::custom(ApiError::InternalError))?;

    if !verified {
        return Err(warp::reject::custom(ApiError::Unauthorized));
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "message": "ok" })),
        StatusCode::OK,
    ))
}

/// Handler delete user
pub async fn delete_user_handler(
    id: i32,
    pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let rows = db::delete_user(&pool, id)
        .await
        .map_err(|_| warp::reject::custom(ApiError::InternalError))?;

    if rows == 0 {
        return Err(warp::reject::custom(ApiError::NotFound));
    }

    Ok(warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT))
}
