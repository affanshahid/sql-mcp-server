use anyhow::{Result, anyhow};
use serde_json::Value;
use sqlx::{
    AssertSqlSafe, mysql::MySqlPoolOptions, postgres::PgPoolOptions, sqlite::SqlitePoolOptions,
};
use sqlx_json::RowExt;
use url::Url;

pub enum DatabasePool {
    MySql(sqlx::MySqlPool),
    Postgres(sqlx::PgPool),
    Sqlite(sqlx::SqlitePool),
}

impl DatabasePool {
    pub async fn connect(url: Url) -> Result<Self> {
        match url.scheme() {
            "mysql" | "mariadb" => Ok(Self::MySql(
                MySqlPoolOptions::new()
                    .max_connections(5)
                    .connect(&url.to_string())
                    .await?,
            )),
            "postgres" | "postgresql" => Ok(Self::Postgres(
                PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&url.to_string())
                    .await?,
            )),
            "sqlite" => Ok(Self::Sqlite(
                SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&url.to_string())
                    .await?,
            )),
            _ => Err(anyhow!("Unsupported database scheme: {}", url.scheme())),
        }
    }

    pub async fn query_as_json(&self, query: &str) -> Result<Vec<Value>> {
        match self {
            Self::MySql(pool) => {
                let rows = sqlx::query(AssertSqlSafe(query)).fetch_all(pool).await?;
                Ok(rows.into_iter().map(|row| row.to_json()).collect())
            }
            Self::Postgres(pool) => {
                let rows = sqlx::query(AssertSqlSafe(query)).fetch_all(pool).await?;
                Ok(rows.into_iter().map(|row| row.to_json()).collect())
            }
            Self::Sqlite(pool) => {
                let rows = sqlx::query(AssertSqlSafe(query)).fetch_all(pool).await?;
                Ok(rows.into_iter().map(|row| row.to_json()).collect())
            }
        }
    }
}
