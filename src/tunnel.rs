use crate::cli::SshOptions;
use anyhow::{Context, Error, Result, anyhow};
use russh::client::Handle;
use russh::{
    client::{self, AuthResult, Config, Handler},
    keys::{self, PrivateKeyWithHashAlg, PublicKey},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tracing::error;
use url::Url;

struct ClientHandler {
    host: String,
    port: u16,
}

impl Handler for ClientHandler {
    type Error = Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        keys::check_known_hosts(&self.host, self.port, server_public_key)
            .context("Unable to verify server key")
    }
}

pub async fn create(opts: SshOptions, db_url: Url) -> Result<(JoinHandle<()>, u16)> {
    let mut session = client::connect(
        Arc::new(Config::default()),
        (opts.host.as_ref(), opts.port),
        ClientHandler {
            host: opts.host.clone(),
            port: opts.port,
        },
    )
    .await?;

    let mut authenticated = false;

    if let Some(password) = opts.password {
        let res = session
            .authenticate_password(opts.username.clone(), password)
            .await?;

        authenticated = matches!(res, AuthResult::Success);
    }

    if let Some(path) = opts.private_key {
        let key = Arc::new(keys::load_secret_key(path, None)?);
        let res = session
            .authenticate_publickey(
                opts.username,
                PrivateKeyWithHashAlg::new(key, session.best_supported_rsa_hash().await?.flatten()),
            )
            .await?;

        authenticated = matches!(res, AuthResult::Success);
    }

    if !authenticated {
        return Err(anyhow!("Failed to authenticate with SSH server"));
    }

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let handle = tokio::spawn(async move {
        let res = listen(listener, session, db_url).await;
        match res {
            Err(err) => error!(?err, "Error while forwarding"),
            _ => (),
        };
    });

    Ok((handle, port))
}

async fn listen(listener: TcpListener, session: Handle<ClientHandler>, db_url: Url) -> Result<()> {
    let session = Arc::new(session);
    let db_url = Arc::new(db_url);

    loop {
        let (con, addr) = listener.accept().await.context("accepting connection")?;
        let cloned_session = Arc::clone(&session);
        let cloned_url = Arc::clone(&db_url);

        tokio::spawn(async move {
            let res = forward(con, addr, cloned_session, cloned_url).await;
            match res {
                Err(err) => error!(?err, "Error while forwarding"),
                _ => (),
            };
        });
    }
}

async fn forward(
    mut con: TcpStream,
    addr: SocketAddr,
    session: Arc<Handle<ClientHandler>>,
    db_url: Arc<Url>,
) -> Result<()> {
    let channel = session
        .channel_open_direct_tcpip(
            db_url.host_str().context("missing host")?,
            db_url.port().context("missing port")? as u32,
            addr.ip().to_string(),
            addr.port() as u32,
        )
        .await?;

    let mut stream = channel.into_stream();

    tokio::io::copy_bidirectional(&mut con, &mut stream).await?;

    Ok(())
}
