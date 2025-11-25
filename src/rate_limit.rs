use std::time::{Duration, SystemTime};
use dashmap::DashMap;
use warp::{Filter, reject};
use crate::errors::ApiError;
use tokio::sync::Mutex;
use std::sync::Arc;

/// Số request tối đa trong WINDOW
const MAX_REQUESTS: usize = 3; // bạn có thể điều chỉnh
/// Kích thước sliding window (giây)
const WINDOW: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct RateLimiter {
    /// Key: ip (String) -> value: Arc<Mutex<Vec<SystemTime>>> (danh sách các timestamp request gần đây)
    pub store: Arc<DashMap<String, Arc<Mutex<Vec<SystemTime>>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            store: Arc::new(DashMap::new()),
        }
    }

    /// Kiểm tra rate limit cho ip; trả Err(ApiError) nếu vượt
    pub async fn check(&self, ip: String) -> Result<(), ApiError> {
        let now = SystemTime::now();

        // Lấy entry (hoặc insert mới với vec rỗng)
        let entry = self.store.entry(ip.clone())
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
            .clone();

        // Lock danh sách timestamp
        let mut guard = entry.lock().await;
        let timestamps = &mut *guard;

        // Loại bỏ timestamp cũ hơn WINDOW
        let window_start = now.checked_sub(WINDOW).unwrap_or(SystemTime::UNIX_EPOCH);
        // retain only timestamps >= window_start
        timestamps.retain(|&t| t >= window_start);

        // Nếu đã đạt giới hạn thì reject
        if timestamps.len() >= MAX_REQUESTS {
            return Err(ApiError::BadRequest(format!(
                "Too many requests from {}. Only {} requests per {} seconds allowed.",
                ip, MAX_REQUESTS, WINDOW.as_secs()
            )));
        }

        // Thêm timestamp hiện tại
        timestamps.push(now);

        // (Tuỳ chọn) log cho dev
        println!("[RateLimit][sliding] IP: {}, count_in_window: {}", ip, timestamps.len());

        Ok(())
    }
}

/// Warp filter để sử dụng trong routes.
/// Vẫn giữ API giống trước: with_rate_limit(limiter)
pub fn with_rate_limit(
    limiter: RateLimiter,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::addr::remote()
        .and_then(move |addr: Option<std::net::SocketAddr>| {
            let limiter = limiter.clone();
            async move {
                let ip = addr
                    .map(|a| a.ip().to_string())
                    .unwrap_or_else(|| "unknown".into());

                limiter.check(ip).await.map_err(|e| reject::custom(e))?;

                Ok::<(), warp::Rejection>(())
            }
        })
        .untuple_one()
}
