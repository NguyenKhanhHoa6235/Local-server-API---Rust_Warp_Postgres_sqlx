use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::Error as JwtError, errors::ErrorKind};
use chrono::{Utc, Duration};
use std::env;

use std::collections::HashMap;
use tokio::sync::Mutex;

use lazy_static::lazy_static;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub name: String,
    pub exp: usize,
}

// Session timeout sau 30 phút không hoạt động
pub const SESSION_TIMEOUT_SECS: i64 = 30*60;

// Lưu last activity cho mỗi user_id
lazy_static! {
    pub static ref LAST_ACTIVITY: Mutex<HashMap<i32, i64>> = Mutex::new(HashMap::new());
}

/// Tạo JWT token
pub fn create_token(user_id: i32, username: &str) -> Result<String, JwtError> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        name: username.to_string(),
        exp,
    };

    // Sau khi login, lưu lại thời gian hoạt động
    tokio::spawn(async move {
        let mut map = LAST_ACTIVITY.lock().await;
        map.insert(user_id, Utc::now().timestamp());
    });

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

/// Xác thực token và kiểm tra session timeout
pub async fn verify_token(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    )
    .map_err(|err| match *err.kind() {
        ErrorKind::ExpiredSignature => "Token expired".to_string(),
        _ => "Token invalid".to_string(),
    })?
    .claims;

    // ---- CHECK SESSION TIMEOUT ----
    let mut map = LAST_ACTIVITY.lock().await;
    let now = Utc::now().timestamp();

    if let Some(last) = map.get(&claims.sub) {
        if now - *last > SESSION_TIMEOUT_SECS {
            map.remove(&claims.sub);
            return Err("Session expired due to inactivity (30 minutes)".into());
        }
    }

    // Mỗi request thành công → server cập nhật last_activity
    map.insert(claims.sub, now);

    Ok(claims)
}
