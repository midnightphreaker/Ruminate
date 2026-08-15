use anyhow::Context;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruminate=info,tower_http=info".into()),
        )
        .init();

    if std::env::var("RUMINATE_TRANSPORT").as_deref() == Ok("stdio") {
        serve_stdio().await
    } else {
        ruminate::http::serve().await
    }
}

async fn serve_stdio() -> anyhow::Result<()> {
    tracing::info!("Ruminate running on stdio");
    let service = ruminate::server::RuminateServer::new()
        .serve(rmcp::transport::stdio())
        .await
        .context("stdio MCP server failed to initialize")?;
    service
        .waiting()
        .await
        .context("stdio MCP transport task failed")?;
    Ok(())
}
