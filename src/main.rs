use anyhow::Result;
use clap::Parser;
use sql_mcp_server::{cli::Cli, tunnel};
use tokio::signal;
use tracing::{Level, debug, error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Cli::parse();
    debug!("{:#?}", args);

    let (tunnel_handle, port) = tunnel::create(args.ssh.unwrap(), args.database_url).await?;
    info!("{port}");

    tokio::select! {
        _ = signal::ctrl_c() => (),
        res = tunnel_handle =>match res {
            Ok(_) => info!("Tunnel closed"),
            Err(e) => error!("Tunnel panicked: {e}"),
        }
    };

    Ok(())
}
