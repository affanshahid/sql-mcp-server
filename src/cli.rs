use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};
use url::Url;

#[derive(ValueEnum, Debug, Clone)]
pub enum Operation {
    Select,
    Insert,
    Update,
    Delete,
    Ddl,
}

#[derive(Debug, Args)]
#[group(requires_all=["host", "username"])]
pub struct SshOptions {
    /// SSH host to connect to
    #[arg(required = false, short = 'H', long, env = "SSH_HOST")]
    pub host: String,

    /// SSH port to connect to
    #[arg(
        required = false,
        short = 'P',
        long,
        default_value_t = 22,
        env = "SSH_PORT"
    )]
    pub port: u16,

    /// SSH username to use for authentication
    #[arg(required = false, short, long, env = "SSH_USERNAME")]
    pub username: String,

    /// SSH password to use for authentication
    #[arg(short, long, env = "SSH_PASSWORD")]
    pub password: Option<String>,

    /// SSH private key to use for authentication
    #[arg(short = 'i', long, env = "SSH_PRIVATE_KEY")]
    pub private_key: Option<PathBuf>,
}

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
pub struct Cli {
    /// The URL of the database to connect to
    ///
    /// Supports: MySQL, MariaDB, Sqlite, Postgres
    #[arg(short, long, env = "DATABASE_URL")]
    pub database_url: Url,

    /// Operations to allow access to
    #[arg(
        short,
        long,
        value_enum,
        value_delimiter=',',
        default_values_t = vec![Operation::Select],
        env="DATABASE_OPERATIONS"
    )]
    pub operations: Vec<Operation>,

    /// SSH tunnel configuration
    #[command(flatten)]
    pub ssh: Option<SshOptions>,
}
