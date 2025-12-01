use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Request body cho /register
#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub name: String,
    pub password: String,
}

// Request body cho /login
#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

// Response cho /register
#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AvatarResponse {
    pub path: String,
}