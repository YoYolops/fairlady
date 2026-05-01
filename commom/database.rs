use crate::constants::SYSTEM_DATABASE_PATH;
use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{self, FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct HistoryRecord {
    pub cid: String,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool, // Implements Arc internally. Thread safe for reading.
}

impl Database {
    pub async fn build(existent_pool: Option<SqlitePool>) -> Result<Self> {
        match existent_pool {
            Some(pool) => Ok(Database { pool: pool }),
            None => {
                let connection_options = SqliteConnectOptions::new()
                    .filename(SYSTEM_DATABASE_PATH)
                    .create_if_missing(true);
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect_with(connection_options)
                    .await?;
                Ok(Database { pool: pool })
            }
        }
    }

    pub async fn get_last_history_record(&self) -> Result<Option<HistoryRecord>> {
        let record = sqlx::query_as::<_, HistoryRecord>(
            "SELECT * FROM history WHERE timestamp = (SELECT MAX(timestamp) FROM history) LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(record)
    }

    pub async fn add_to_history(&self, cid: &str) -> Result<()> {
        sqlx::query("INSERT INTO history (cid) VALUES (?)")
            .bind(cid)
            .execute(&self.pool)
            .await
            .context("FAILED inserting data into history")?;
        Ok(())
    }
}
