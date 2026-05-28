use crate::constants::SYSTEM_DATABASE_PATH;
use anyhow::{Context, Result, bail};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{self, FromRow, SqlitePool};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub enum Operation {
    Encryption,
    Decryption,
}

pub struct PerformancePoint {
    pub strategy: String,
    pub init_timestamp: Option<i64>,
    pub final_timestamp: Option<i64>,
    pub operation: Operation,
    pub payload_size: i64,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Encryption => write!(f, "encryption"),
            Operation::Decryption => write!(f, "decryption"),
        }
    }
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
                timestamp: db_record
                    .timestamp
                    .parse()
                    .expect("Failed to parse timestamp data to u128"),
            })),
            None => Ok(None),
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

    pub async fn add_perf_point(&self, perf_point: PerformancePoint) -> Result<()> {
        sqlx::query(
            "INSERT INTO perf_points (strategy, init_timestamp, final_timestamp, operation, payload_size) VALUES (?, ?, ?, ?)")
            .bind(perf_point.strategy)
            .bind(perf_point.init_timestamp)
            .bind(perf_point.final_timestamp)
            .bind(perf_point.operation.to_string())
            .bind(perf_point.payload_size)
            .execute(&self.pool)
            .await
            .context("FAILED inserting data into performance point")?;
        Ok(())
    }
}

impl PerformancePoint {
    pub fn clock_in(&mut self) -> Result<()> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                self.init_timestamp = Some(duration.as_nanos() as i64);
                Ok(())
            }
            Err(_) => bail!("Failed while getting unix timestamp for clock in"),
        }
    }

    pub fn clock_out(&mut self) -> Result<()> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                self.final_timestamp = Some(duration.as_nanos() as i64);
                Ok(())
            }
            Err(_) => bail!("Failed while getting unix timestamp for clock out"),
        }
    }
}
