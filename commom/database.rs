use crate::constants::SYSTEM_DATABASE_PATH;
use anyhow::{Context, Ok, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{self, FromRow, SqlitePool};

#[derive(Debug, FromRow)]
struct HistoryDBRecord {
    pub cid: String,
    pub timestamp: String,
}

pub struct HistoryRecord {
    pub cid: String,
    pub timestamp: u128,
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
        let record = sqlx::query_as::<_, HistoryDBRecord>(
            "SELECT * FROM upload_history WHERE timestamp = (SELECT MAX(CAST(timestamp AS INTEGER)) FROM upload_history) LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        match record {
            Some(db_record) => Ok(Some(HistoryRecord {
                cid: db_record.cid,
                timestamp: db_record.timestamp.parse().expect("Failed to parse timestamp data to u128"),
            })),
            None => Ok(None)
        }
    }

    pub async fn add_to_history(&self, cid: &str, nano_timestamp: &u128) -> Result<()> {
        sqlx::query("INSERT INTO upload_history (cid, timestamp) VALUES (?, ?)")
            .bind(cid)
            .bind(nano_timestamp.to_string())
            .execute(&self.pool)
            .await
            .context("FAILED inserting data into history")?;
        Ok(())
    }
}
