use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
// use time::OffsetDateTime;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
