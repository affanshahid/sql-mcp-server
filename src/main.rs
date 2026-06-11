use anyhow::{Result, anyhow};
use clap::Parser;
use sql_mcp_server::{cli::Cli, tunnel};
use sqlx::any::AnyPoolOptions;
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

    sqlx::any::install_default_drivers();

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&url.to_string())
        .await?;

    let (res,): (i64,) = sqlx::query_as("Select 67").fetch_one(&pool).await?;
    info!("Response: {res}");

    match handle {
        Some(handle) => tokio::select! {
            _ = signal::ctrl_c() => (),
            res = handle => match res {
                Ok(_) => info!("Tunnel closed"),
                Err(e) => error!("Tunnel panicked: {e}"),
            }
        },
        None => signal::ctrl_c().await?,
    }

    Ok(())
}
