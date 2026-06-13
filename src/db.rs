use anyhow::{Result, anyhow};
use serde_json::Value;
use sqlparser::{
    ast::Statement,
    dialect::{Dialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect},
    parser::Parser,
};
use sqlx::{
    AssertSqlSafe, mysql::MySqlPoolOptions, postgres::PgPoolOptions, sqlite::SqlitePoolOptions,
};
use sqlx_json::RowExt;
use url::Url;

use crate::cli::Operation;

pub enum DatabasePool {
    MySql(sqlx::MySqlPool),
    Postgres(sqlx::PgPool),
    Sqlite(sqlx::SqlitePool),
}

pub enum OperationInfo {
    Select { has_limit: bool },
    Insert,
    Update { has_where: bool },
    Delete { has_where: bool },
    Ddl,
}

impl OperationInfo {
    pub fn operation(&self) -> Operation {
        match self {
            OperationInfo::Select { .. } => Operation::Select,
            OperationInfo::Insert => Operation::Insert,
            OperationInfo::Update { .. } => Operation::Update,
            OperationInfo::Delete { .. } => Operation::Delete,
            OperationInfo::Ddl => Operation::Ddl,
        }
    }
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

    pub fn parse_operations(&self, query: &str) -> Result<Vec<OperationInfo>> {
        let dialect: &dyn Dialect = match self {
            DatabasePool::MySql(_) => &MySqlDialect {},
            DatabasePool::Postgres(_) => &PostgreSqlDialect {},
            DatabasePool::Sqlite(_) => &SQLiteDialect {},
        };

        let ast = Parser::parse_sql(dialect, query)?;

        Ok(ast
            .into_iter()
            .map(|stmt| match stmt {
                Statement::Query(query) => {
                    let has_limit = query.limit_clause.is_some();
                    OperationInfo::Select { has_limit }
                }
                Statement::Insert(_) => OperationInfo::Insert,
                Statement::Update(update) => {
                    let has_where = update.selection.is_some();
                    OperationInfo::Update { has_where }
                }
                Statement::Delete(delete) => {
                    let has_where = delete.selection.is_some();
                    OperationInfo::Delete { has_where }
                }
                _ => OperationInfo::Ddl,
            })
            .collect())
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
