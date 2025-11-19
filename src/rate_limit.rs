use std::time::{Duration, SystemTime};
use dashmap::DashMap;
use warp::{Filter, reject};
use crate::errors::ApiError;
use tokio::sync::Mutex;
use std::sync::Arc;

const MAX_REQUESTS: u32 = 3; // tối đa request mỗi WINDOW
const WINDOW: Duration = Duration::from_secs(60); // thời gian window (giây)

#[derive(Clone)]
pub struct RateLimiter {
    pub store: Arc<DashMap<String, Arc<Mutex<(u32, SystemTime)>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            store: Arc::new(DashMap::new()),
        }
    }

    pub async fn check(&self, ip: String) -> Result<(), ApiError> {
        let now = SystemTime::now();

        let entry = self.store.entry(ip.clone())
            .or_insert_with(|| Arc::new(Mutex::new((0, now))))
            .clone();

        let mut guard = entry.lock().await;
        let (ref mut count, ref mut start_time) = *guard;

        if now.duration_since(*start_time).unwrap_or(Duration::from_secs(0)) > WINDOW {
            *count = 0;
            *start_time = now;
        }

        if *count >= MAX_REQUESTS {
            return Err(ApiError::BadRequest(format!(
                "Too many requests from {}. Only {} requests per {} seconds allowed.",
                ip, MAX_REQUESTS, WINDOW.as_secs()
            )));
        }

        *count += 1;
        println!("[RateLimit] IP: {}, count: {}", ip, *count);

        Ok(())
    }
}

pub fn with_rate_limit(
    limiter: RateLimiter,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::addr::remote()
        .and_then(move |addr: Option<std::net::SocketAddr>| {
            let limiter = limiter.clone();
            async move {
                let ip = addr
                    .map(|a| a.ip().to_string())
                    .unwrap_or("unknown".into());

                limiter.check(ip).await.map_err(|e| reject::custom(e))?;

                Ok::<(), warp::Rejection>(())
            }
        })
        .untuple_one()
}
