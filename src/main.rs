use std::io;

use anyhow::{Result, anyhow};
use clap::Parser;
use rmcp::{ServiceExt, transport};
use sql_mcp_server::{cli::Cli, db::DatabasePool, mcp::SqlMcpServer, tunnel};
use tracing::{Level, debug, error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    debug!("Args: {:#?}", args);

    let (url, handle) = match args.ssh {
        Some(ssh_opts) => {
            let mut forwarding_url = args.database_url.clone();
            let (tunnel_handle, port) = tunnel::create(ssh_opts, args.database_url).await?;
            forwarding_url.set_host(Some("127.0.0.1"))?;
            forwarding_url
                .set_port(Some(port))
                .map_err(|_| anyhow!("cannot be base"))?;

            debug!("Tunnel listening on 127.0.0.1:{port}");
            (forwarding_url, Some(tunnel_handle))
        }
        None => (args.database_url, None),
    };

    let pool = DatabasePool::connect(url).await?;
    let server = SqlMcpServer::new(pool, args.permissions);
    let service = server.serve(transport::stdio()).await?;

    match handle {
        Some(handle) => tokio::select! {
            _ = service.waiting() => (),
            res = handle => match res {
                Ok(_) => info!("Tunnel closed"),
                Err(e) => error!("Tunnel panicked: {e}"),
            }
        },
        None => {
            service.waiting().await?;
        }
    }

    Ok(())
}
