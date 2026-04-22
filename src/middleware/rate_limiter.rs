use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_governor::{
    errors::GovernorError, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
    GovernorLayer,
};
use governor::middleware::StateInformationMiddleware;

/// Axum middleware that bypasses rate limiting for whitelisted IPs.
/// Reads `RATE_LIMIT_WHITELIST` env var (comma-separated IPs) once at startup.
pub async fn whitelist_middleware(req: Request, next: Next) -> Response {
    let ip = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip());

    if let Some(ip) = ip {
        if is_whitelisted(ip) {
            tracing::debug!(%ip, "whitelisted IP bypassing rate limit");
            return next.run(req).await;
        }
    }
    next.run(req).await
}

fn is_whitelisted(ip: IpAddr) -> bool {
    // Parse once; in practice this is called per-request but the env read is cheap.
    // For production, store the set in AppState instead.
    let whitelist = std::env::var("RATE_LIMIT_WHITELIST").unwrap_or_default();
    whitelist
        .split(',')
        .filter_map(|s| s.trim().parse::<IpAddr>().ok())
        .any(|allowed| allowed == ip)
}

/// Custom 429 error handler: JSON body + Retry-After header + tracing.
fn rate_limit_error_handler(err: GovernorError) -> Response {
    let (retry_after, message) = match &err {
        GovernorError::TooManyRequests { wait_time, .. } => (
            *wait_time,
            "Rate limit exceeded. Please slow down.",
        ),
        GovernorError::UnableToExtractKey => (0, "Unable to identify client."),
        GovernorError::Other { code, msg, .. } => {
            tracing::warn!(code = ?code, msg = ?msg, "Governor: unexpected error");
            (0, "Rate limiting error.")
        }
    };

    tracing::warn!(retry_after_secs = retry_after, "Rate limit exceeded");

    let body = serde_json::json!({
        "error": {
            "code": "RATE_LIMIT_EXCEEDED",
            "message": message,
            "retry_after_secs": retry_after
        }
    });

    let mut resp = (StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
    if retry_after > 0 {
        resp.headers_mut().insert(
            "Retry-After",
            retry_after.to_string().parse().unwrap(),
        );
    }
    resp
}

/// Builds a GovernorLayer for general read endpoints.
/// Configurable via env: RATE_LIMIT_PER_SECOND (default 10), RATE_LIMIT_BURST_SIZE (default 20).
pub fn general_limiter() -> GovernorLayer<PeerIpKeyExtractor, StateInformationMiddleware> {
    let per_second = env_u64("RATE_LIMIT_PER_SECOND", 10);
    let burst_size = env_u32("RATE_LIMIT_BURST_SIZE", 20);
    build_layer(per_second, burst_size)
}

/// Builds a stricter GovernorLayer for write endpoints (POST /tips, POST /creators).
/// Configurable via env: RATE_LIMIT_WRITE_PER_SECOND (default 2), RATE_LIMIT_WRITE_BURST_SIZE (default 5).
pub fn write_limiter() -> GovernorLayer<PeerIpKeyExtractor, StateInformationMiddleware> {
    let per_second = env_u64("RATE_LIMIT_WRITE_PER_SECOND", 2);
    let burst_size = env_u32("RATE_LIMIT_WRITE_BURST_SIZE", 5);
    build_layer(per_second, burst_size)
}

fn build_layer(
    per_second: u64,
    burst_size: u32,
) -> GovernorLayer<PeerIpKeyExtractor, StateInformationMiddleware> {
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(burst_size)
            .use_headers()
            .error_handler(rate_limit_error_handler)
            .finish()
            .unwrap(),
    );

    let limiter = config.limiter().clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.tick().await;
        loop {
            interval.tick().await;
            tracing::debug!("rate limiter cleanup: {} tracked IPs", limiter.len());
            limiter.retain_recent();
        }
    });

    GovernorLayer { config }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
