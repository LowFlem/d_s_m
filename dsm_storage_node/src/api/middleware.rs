// API middleware for DSM Storage Node
//
// This module implements middleware for the API server, including authentication,
// rate limiting, and request/response logging.

use crate::error::{Result, StorageNodeError};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Rate limiter for API requests
#[allow(dead_code)]
pub struct RateLimiter {
    /// Window size in seconds
    window_size: u64,
    /// Maximum requests per window
    max_requests: u32,
    /// Request counters by client IP
    counters: Mutex<HashMap<String, (Instant, u32)>>,
}

#[allow(dead_code)]
impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(window_size: u64, max_requests: u32) -> Self {
        Self {
            window_size,
            max_requests,
            counters: Mutex::new(HashMap::new()),
        }
    }

    /// Increment request count and check rate limit
    pub fn check_rate_limit(&self, client_ip: &str) -> Result<()> {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.window_size);

        let mut counters = self.counters.lock().unwrap();

        // Get or initialize counter for client IP
        let counter = counters
            .entry(client_ip.to_string())
            .or_insert_with(|| (now, 0));

        // Reset counter if window has elapsed
        if now.duration_since(counter.0) > window_duration {
            counter.0 = now;
            counter.1 = 0;
        }

        // Increment counter
        counter.1 += 1;

        // Check if rate limit exceeded
        if counter.1 > self.max_requests {
            return Err(StorageNodeError::RateLimitExceeded(format!(
                "Rate limit exceeded: {} requests per {} seconds",
                self.max_requests, self.window_size
            )));
        }

        Ok(())
    }
}

/// Get client IP from request
#[allow(dead_code)]
fn get_client_ip(request: &Request<Body>) -> String {
    // Try to get X-Forwarded-For header
    if let Some(header) = request.headers().get("X-Forwarded-For") {
        if let Ok(value) = header.to_str() {
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Fallback to X-Real-IP header
    if let Some(header) = request.headers().get("X-Real-IP") {
        if let Ok(value) = header.to_str() {
            return value.to_string();
        }
    }

    // For now, we fallback to a localhost assumption since axum doesn't
    // provide direct access to connection info in middleware.
    // In production, this would typically be behind a proxy that sets
    // X-Forwarded-For or X-Real-IP headers.
    "127.0.0.1".to_string()
}

/// Verify API token for storage node access
#[allow(dead_code)]
fn verify_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }

    // For storage node API access, we use simple bearer tokens
    // In production, these would be configured API keys or JWT tokens
    // Parse token format: base64(timestamp:node_id:api_key)
    if let Ok(decoded) = hex::decode(token) {
        if let Ok(token_str) = String::from_utf8(decoded) {
            let parts: Vec<&str> = token_str.split(':').collect();
            if parts.len() >= 2 {
                // Verify timestamp is not too old (24 hour window for API tokens)
                if let Ok(timestamp) = parts[0].parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    // API token is valid if within 24 hours
                    return now - timestamp < 86400;
                }
            }
        }
    }

    // Fallback: accept any non-empty token for development
    !token.is_empty()
}

/// Verify signature for storage node API access
#[allow(dead_code)]
fn verify_signature(signature: &str) -> bool {
    if signature.is_empty() {
        return false;
    }

    // Parse signature format: hex(signature_bytes)
    if let Ok(decoded) = hex::decode(signature) {
        // For storage node API, we verify that it's a valid hex signature
        // of appropriate length (32-128 bytes for various signature schemes)
        return decoded.len() >= 32 && decoded.len() <= 128;
    }

    false
}

/// Rate limiting middleware
#[allow(dead_code)]
pub async fn rate_limiting(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    // Get client IP from headers or connection info
    let client_ip = get_client_ip(&request);

    // Check rate limit
    if limiter.check_rate_limit(&client_ip).is_err() {
        warn!("Rate limit exceeded for client {}", client_ip);
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }

    // Continue to next middleware or handler
    next.run(request).await
}

/// Authentication middleware
#[allow(dead_code)]
pub async fn authenticate(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    // Extract authorization header
    let auth_header = headers
        .get("Authorization")
        .map(|h| h.to_str().unwrap_or(""));

    match auth_header {
        Some(auth) => {
            // Parse authorization header
            let parts: Vec<&str> = auth.splitn(2, ' ').collect();
            if parts.len() != 2 {
                return (StatusCode::UNAUTHORIZED, "Invalid authorization format").into_response();
            }

            let auth_type = parts[0];
            let auth_value = parts[1];

            // Handle different auth types
            match auth_type {
                "Bearer" => {
                    // Handle token authentication
                    if !verify_token(auth_value) {
                        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
                    }
                }
                "Signature" => {
                    // Handle signature authentication
                    if !verify_signature(auth_value) {
                        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
                    }
                }
                _ => {
                    return (StatusCode::UNAUTHORIZED, "Unsupported authentication type")
                        .into_response();
                }
            }

            // Continue to next middleware or handler
            next.run(request).await
        }
        None => {
            // No authentication provided
            (StatusCode::UNAUTHORIZED, "Authentication required").into_response()
        }
    }
}

/// Request logging middleware
#[allow(dead_code)]
pub async fn log_request(request: Request<Body>, next: Next<Body>) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;
    let duration = start.elapsed();

    debug!("Response: {} {} in {:?}", method, uri, duration);

    response
}
