use sea_orm::DatabaseConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

use crate::error::AppResult;

pub async fn create_pool(db_path: &str) -> AppResult<DatabaseConnection> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let db = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);

    Ok(db)
}
