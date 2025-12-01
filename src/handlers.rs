use crate::models::{RegisterRequest, LoginRequest, UserResponse, AvatarResponse};
use crate::errors::ApiError;
use crate::db;
use crate::jwt;
use sqlx::PgPool;
use warp::http::StatusCode;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use chrono::Utc;
use warp::Buf;

/// Root handler
pub async fn root_handler() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "message": "Welcome to Local Server API - Rust!"
    })))
}

/// Register handler
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

/// Login handler
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

/// Delete user handler
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

/// Upload avatar handler
pub async fn upload_avatar_handler(
    id: i32,
    pool: PgPool,
    claims: crate::jwt::Claims,
    mut form: warp::multipart::FormData,
) -> Result<impl warp::Reply, warp::Rejection> {
    if claims.sub != id {
        return Err(warp::reject::custom(ApiError::NotAllowed));
    }

    let uploads_dir = "./uploads";
    if !Path::new(uploads_dir).exists() {
        tokio::fs::create_dir_all(uploads_dir)
            .await
            .map_err(|e| warp::reject::custom(ApiError::InternalError(format!("Failed create uploads dir: {}", e))))?;
    }

    let mut saved_path: Option<String> = None;

    while let Some(part) = form.next().await {
        let part = part.map_err(|e| warp::reject::custom(ApiError::InternalError(format!("Multipart error: {}", e))))?;
        if part.name() != "avatar" { continue; }

        let orig_filename = part.filename().unwrap_or("avatar");
        let ext = Path::new(orig_filename).extension().and_then(|e| e.to_str()).unwrap_or("png");
        let timestamp = Utc::now().timestamp();
        let filename = format!("user_{}_{}.{}", id, timestamp, ext);
        let filepath = format!("{}/{}", uploads_dir, filename);

        let bytes: Vec<u8> = part.stream()
            .try_fold(Vec::new(), |mut acc, buf| async move {
                acc.extend_from_slice(buf.chunk());
                Ok(acc)
            })
            .await
            .map_err(|e| warp::reject::custom(ApiError::InternalError(format!("Stream error: {}", e))))?;

        let mut file = tokio::fs::File::create(&filepath)
            .await
            .map_err(|e| warp::reject::custom(ApiError::InternalError(format!("File create error: {}", e))))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| warp::reject::custom(ApiError::InternalError(format!("File write error: {}", e))))?;

        saved_path = Some(filepath.clone());
    }

    let saved_path = saved_path.ok_or_else(|| warp::reject::custom(ApiError::BadRequest("No 'avatar' file found".into())))?;
    db::update_user_avatar(&pool, id, &saved_path).await.map_err(warp::reject::custom)?;

    Ok(warp::reply::with_status(
        warp::reply::json(&AvatarResponse { path: saved_path }),
        StatusCode::OK
    ))
}

/// Get avatar handler
pub async fn get_avatar_handler(id: i32, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    let avatar_path = db::get_avatar_path(&pool, id)
        .await
        .map_err(warp::reject::custom)?
        .ok_or_else(|| warp::reject::custom(ApiError::NotFound))?;

    let data = tokio::fs::read(&avatar_path)
        .await
        .map_err(|_| warp::reject::custom(ApiError::NotFound))?;

    let content_type = match Path::new(&avatar_path).extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
        Some(ext) if ext == "png" => "image/png",
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "gif" => "image/gif",
        Some(ext) if ext == "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    Ok(warp::reply::with_header(data, "content-type", content_type))
}