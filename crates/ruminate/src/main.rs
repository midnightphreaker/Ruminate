mod http;
mod models;
mod reflect;
mod server;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruminate=info,tower_http=info".into()),
        )
        .init();

    http::serve().await
}
