use anyhow::{Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use commom::constants::{SYSTEM_DATABASE_PATH, SYSTEM_FOREIGN_DATA_PATH};
use tokio::fs;

pub async fn system_startup() -> Result<()> {
    fs::create_dir_all(SYSTEM_FOREIGN_DATA_PATH).await?; // ensures folder existence
    init_db().await?;
    Ok(())
}

async fn init_db() -> Result<SqlitePool> {
    let connection_options = SqliteConnectOptions::new()
        .filename(SYSTEM_DATABASE_PATH)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await?;
    sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                cid TEXT PRIMARY KEY,
                -- Stores the exact seconds since the Unix Epoch:
                timestamp INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
            );
            "#,
        )
        .execute(&pool)
        .await?;
    Ok(pool)
}