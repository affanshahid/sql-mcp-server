use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
enum Operation {
    Select,
    Insert,
    Update,
    Delete,
    Ddl,
}

#[derive(Debug, Args)]
#[group(requires_all=["host", "port", "username"])]
struct SshOptions {
    /// SSH host to connect to
    #[arg(required = false, short = 'H', long, env = "SSH_HOST")]
    host: String,

    /// SSH port to connect to
    #[arg(
        required = false,
        short = 'P',
        long,
        default_value_t = 22,
        env = "SSH_PORT"
    )]
    port: u16,

    /// SSH username to use for authentication
    #[arg(required = false, short, long, env = "SSH_USERNAME")]
    username: String,

    /// SSH password to use for authentication
    #[arg(short, long, env = "SSH_PASSWORD")]
    password: Option<String>,

    /// SSH private key to use for authentication
    #[arg(short = 'i', long, env = "SSH_PRIVATE_KEY")]
    private_key: Option<PathBuf>,
}

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
struct Cli {
    /// The URL of the database to connect to
    ///
    /// Supports: MySQL, Sqlite, Postgres, MSSQL
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: String,

    /// Operations to allow access to
    #[arg(
        short,
        long,
        value_enum,
        value_delimiter=',',
        default_values_t = vec![Operation::Select],
        env="DATABASE_OPERATIONS"
    )]
    operations: Vec<Operation>,

    /// SSH tunnel configuration
    #[command(flatten)]
    ssh: Option<SshOptions>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    println!("{:#?}", args);
}
