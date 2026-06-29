use std::net::SocketAddr;

use anyhow::Context;
use axum::{Json, Router, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, json};

use crate::server::RuminateServer;

pub fn app() -> Router {
    let mcp_service = StreamableHttpService::new(
        || Ok(RuminateServer::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    Router::new()
        .route("/healthz", get(healthz))
        .nest_service("/mcp", mcp_service)
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
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .context("HTTP server failed")
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
}
