use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::TryStreamExt;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    proxy_key: Option<String>,
}

#[derive(Deserialize)]
struct ProxyQuery {
    url: Option<String>,
    key: Option<String>,
}

const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
const ACCEPT_HEADER: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let proxy_key = env::var("PROXY_KEY").ok();

    // High performance reqwest client with connection pooling & keep-alive
    let client = reqwest::Client::builder()
        .pool_idle_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(30)
        .tcp_keepalive(Duration::from_secs(15))
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .expect("Failed to build HTTP client");

    let state = Arc::new(AppState { client, proxy_key });

    let app = Router::new()
        .route("/", get(handle_proxy))
        .route("/proxy", get(handle_proxy))
        .route("/health", get(handle_health))
        .route("/ping", get(handle_health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("[Rust Proxy] Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "engine": "rust-axum-1.75+",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

async fn handle_proxy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ProxyQuery>,
) -> Response {
    // 1. Optional API Key Check
    if let Some(ref required_key) = state.proxy_key {
        let provided_key = headers
            .get("x-proxy-key")
            .and_then(|v| v.to_str().ok())
            .or(query.key.as_deref());

        if provided_key != Some(required_key.as_str()) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Unauthorized: Invalid or missing PROXY_KEY" })),
            )
                .into_response();
        }
    }

    // 2. Validate URL
    let target_url = match query.url {
        Some(ref u) if u.starts_with("http://") || u.starts_with("https://") => u.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Missing or invalid 'url' query parameter",
                    "usage": "GET /?url=https://hubcloud.pw/drive/..."
                })),
            )
                .into_response();
        }
    };

    // 3. Send upstream request with connection reuse
    let upstream_req = state
        .client
        .get(&target_url)
        .header(USER_AGENT, BROWSER_UA)
        .header(ACCEPT, ACCEPT_HEADER)
        .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
        .header(REFERER, &target_url)
        .header(COOKIE, "xla=s4t; xyt=1")
        .header("Sec-Ch-Ua", "\"Chromium\";v=\"128\", \"Not;A=Brand\";v=\"24\", \"Google Chrome\";v=\"128\"")
        .header("Sec-Ch-Ua-Mobile", "?0")
        .header("Sec-Ch-Ua-Platform", "\"Windows\"")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Upgrade-Insecure-Requests", "1");

    match upstream_req.send().await {
        Ok(upstream_resp) => {
            let status = StatusCode::from_u16(upstream_resp.status().as_u16())
                .unwrap_or(StatusCode::OK);

            let mut resp_headers = HeaderMap::new();
            if let Some(ct) = upstream_resp.headers().get(reqwest::header::CONTENT_TYPE) {
                resp_headers.insert(axum::http::header::CONTENT_TYPE, ct.clone());
            } else {
                resp_headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
            }

            resp_headers.insert(
                HeaderValue::from_static("x-upstream-status"),
                HeaderValue::from_str(&upstream_resp.status().as_u16().to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("200")),
            );

            // Stream body directly without buffering
            let body_stream = upstream_resp.bytes_stream().map_err(|e| {
                error!("Stream error: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            });

            let body = Body::from_stream(body_stream);
            (status, resp_headers, body).into_response()
        }
        Err(err) => {
            error!("Upstream fetch failed: {}", err);
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "Failed to fetch upstream URL",
                    "details": err.to_string()
                })),
            )
                .into_response()
        }
    }
}
