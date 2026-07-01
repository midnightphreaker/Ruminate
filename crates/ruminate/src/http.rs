use std::net::SocketAddr;

use anyhow::Context;
use axum::{Json, Router, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, json};

use crate::server::RuminateServer;

pub fn app() -> Router {
    app_with_config(streamable_http_config_from_env())
}

pub fn app_with_config(config: StreamableHttpServerConfig) -> Router {
    let mcp_service = StreamableHttpService::new(
        || Ok(RuminateServer::new()),
        LocalSessionManager::default().into(),
        config,
    );

    Router::new()
        .route("/healthz", get(healthz))
        .nest_service("/mcp", mcp_service)
}

fn streamable_http_config_from_env() -> StreamableHttpServerConfig {
    let mut config = StreamableHttpServerConfig::default();
    if let Ok(value) = std::env::var("RUMINATE_ALLOWED_HOSTS") {
        let hosts: Vec<String> = value
            .split(',')
            .map(str::trim)
            .filter(|host| !host.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        if !hosts.is_empty() {
            let mut allowed_hosts = config.allowed_hosts.clone();
            allowed_hosts.extend(hosts);
            config = config.with_allowed_hosts(allowed_hosts);
        }
    }
    config
}

pub async fn healthz() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "ruminate",
    }))
}

pub async fn serve() -> anyhow::Result<()> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!("Ruminate running on streamable HTTP at /mcp port {port}");
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server failed")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "failed to install Ctrl-C signal handler");
        }
    };

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let terminate = async {
            match signal(SignalKind::terminate()) {
                Ok(mut stream) => {
                    stream.recv().await;
                }
                Err(error) => {
                    tracing::warn!(%error, "failed to install SIGTERM signal handler");
                    std::future::pending::<()>().await;
                }
            }
        };

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await;
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn healthz_returns_ok() {
        let response = app()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json, json!({ "status": "ok", "service": "ruminate" }));
    }

    #[tokio::test]
    async fn configured_non_loopback_host_is_not_rejected() {
        let response = app_with_config(StreamableHttpServerConfig::default().with_allowed_hosts([
            "localhost",
            "127.0.0.1",
            "::1",
            "10.0.0.2",
            "10.0.0.2:8000",
        ]))
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/mcp")
                .header("host", "10.0.0.2:8000")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

        assert_ne!(response.status(), axum::http::StatusCode::FORBIDDEN);
    }
}
