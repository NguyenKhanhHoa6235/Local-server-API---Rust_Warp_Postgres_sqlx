use warp::Filter;
use sqlx::PgPool;
use crate::handlers;
use crate::models::{RegisterRequest, LoginRequest};

pub fn create_routes(pool: PgPool) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let db_filter = warp::any().map(move || pool.clone());

    // Route root GET "/"
    let root = warp::path::end()
        .and(warp::get())
        .and_then(handlers::root_handler);

    // Route đăng ký user
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json::<RegisterRequest>())
        .and(db_filter.clone())
        .and_then(handlers::register_handler);

    // Route login
    let login = warp::path("login")
        .and(warp::post())
        .and(warp::body::json::<LoginRequest>())
        .and(db_filter.clone())
        .and_then(handlers::login_handler);

    // Route xóa user theo id DELETE "/users/{id}"
    let delete = warp::path!("users" / i32)
        .and(warp::delete())
        .and(db_filter.clone())
        .and_then(handlers::delete_user_handler);

    // Kết hợp tất cả route - xử lý lỗi tùy chỉnh
    let routes = root.or(register).or(login).or(delete)
        .recover(|err: warp::Rejection| async move {
            if let Some(e) = err.find::<crate::errors::ApiError>() {
                let code = e.status_code();
                let msg = serde_json::json!({ "error": e.to_string() });
                Ok(warp::reply::with_status(warp::reply::json(&msg), code))
            } else {
                // Lỗi bất ngờ
                let msg = serde_json::json!({ "error": "internal server error" });
                Ok(warp::reply::with_status(
                    warp::reply::json(&msg),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        });

    routes
}
