use sqlx::PgPool;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::SaltString;
use rand_core::OsRng;
use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::errors::ApiError;

/// Hash mật khẩu bằng Argon2
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(password_hash.to_string())
}

/// Xác thực mật khẩu với hash
pub fn verify_password(hash: &str, password: &str) -> Result<bool> {
    let parsed = password_hash::PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!(e))?;
    let argon2 = Argon2::default();
    // Trả về true nếu verify thành công (is_ok), false nếu thất bại
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}

/// Tạo user mới trong DB
pub async fn create_user(pool: &PgPool, name: &str, hash: &str) -> Result<i32, ApiError> {
    let res = sqlx::query!(
        r#"INSERT INTO users (name, password_hash) VALUES ($1, $2) RETURNING id"#,
        name,
        hash
    )
    .fetch_one(pool)
    .await;

    match res {
        Ok(rec) => Ok(rec.id),
        Err(sqlx::Error::Database(db_err)) => {
            // Lỗi 23505 là lỗi khóa duy nhất (unique constraint violation), ở đây là `name`
            if db_err.code() == Some("23505".into()) { 
                Err(ApiError::UserExists)
            } else {
                Err(ApiError::InternalError(format!("Database error: {}", db_err)))
            }
        }
        Err(e) => Err(ApiError::InternalError(format!("Database error: {}", e))),
    }
}

/// Lấy user theo tên
pub async fn get_user_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<(i32, String, String, DateTime<Utc>)>, ApiError> {
    let rec = sqlx::query!(
        r#"SELECT id, name, password_hash, created_at FROM users WHERE name = $1"#,
        name
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("DB fetch error: {}", e)))?;

    Ok(rec.map(|r| (r.id, r.name, r.password_hash, r.created_at)))
}

/// Xóa user theo ID
pub async fn delete_user(pool: &PgPool, id: i32) -> Result<u64, ApiError> {
    let res = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("DB delete error: {}", e)))?;
    Ok(res.rows_affected())
}