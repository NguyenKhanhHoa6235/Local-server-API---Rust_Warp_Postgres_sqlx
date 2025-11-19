use crate::models::{RegisterRequest, LoginRequest, UserResponse};
use crate::errors::ApiError;
use crate::db;
use crate::jwt;
use sqlx::PgPool;
use warp::http::StatusCode;

pub async fn root_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "message": "Welcome to Local Server API - Rust!"
    })))
}

pub async fn register_handler(body: RegisterRequest, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    if body.name.trim().is_empty() {
        return Err(warp::reject::custom(ApiError::BadRequest("Name cannot be empty".into())));
    }
    if body.password.len() < 8 {
        return Err(warp::reject::custom(ApiError::BadRequest("Password must be at least 8 chars".into())));
    }

    let hash = db::hash_password(&body.password)
        .map_err(|_| warp::reject::custom(ApiError::InternalError("Password hash failed".into())))?;

    let id = db::create_user(&pool, &body.name, &hash)
        .await
        .map_err(warp::reject::custom)?;

    let user_opt = db::get_user_by_name(&pool, &body.name)
        .await
        .map_err(warp::reject::custom)?;

    let (_id, _name, _hash, created_at) =
        user_opt.ok_or_else(|| warp::reject::custom(ApiError::InternalError("User retrieval failed".into())))?;

    let resp = UserResponse { id, name: body.name, created_at };
    Ok(warp::reply::with_status(warp::reply::json(&resp), StatusCode::CREATED))
}

pub async fn login_handler(body: LoginRequest, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    if body.name.trim().is_empty() {
        return Err(warp::reject::custom(ApiError::BadRequest("Name cannot be empty".into())));
    }

    let user_opt = db::get_user_by_name(&pool, &body.name)
        .await
        .map_err(warp::reject::custom)?;

    let (user_id, _name, password_hash, _created_at) = match user_opt {
        Some(u) => u,
        None => return Err(warp::reject::custom(ApiError::Unauthorized("User not found".into()))),
    };

    let verified = db::verify_password(&password_hash, &body.password)
        .map_err(|_| warp::reject::custom(ApiError::InternalError("Password verification failed".into())))?;

    if !verified {
        return Err(warp::reject::custom(ApiError::Unauthorized("Incorrect password".into())));
    }

    let token = jwt::create_token(user_id, &body.name)
        .map_err(|_| warp::reject::custom(ApiError::InternalError("JWT creation failed".into())))?;

    Ok(warp::reply::with_status(warp::reply::json(&serde_json::json!({
        "message": "Login successful",
        "token": token
    })), StatusCode::OK))
}

pub async fn delete_user_handler(id: i32, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    let rows = db::delete_user(&pool, id)
        .await
        .map_err(warp::reject::custom)?;

    if rows == 0 {
        return Err(warp::reject::custom(ApiError::NotFound));
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "message": "User deleted successfully" })),
        StatusCode::OK
    ))
}
