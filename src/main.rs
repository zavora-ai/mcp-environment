use mcp_environment::server::EnvironmentServer;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider().install_default().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr).with_ansi(false).init();
    tracing::info!("Starting Environment / Runtime MCP server");
    let server = EnvironmentServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
