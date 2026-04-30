use anyhow::{Result, Context};
use sqlx::{self, SqlitePool, FromRow};

#[derive(Debug, FromRow)]
pub struct HistoryRecord {
    pub cid: String,
    pub timestamp: i64
}

pub async fn get_last_history_record(pool: &SqlitePool) -> Result<Option<HistoryRecord>> {
    let record = sqlx::query_as::<_, HistoryRecord>(
        "SELECT * FROM history WHERE timestamp = MAX(SELECT timestamp FROM history)"
    )
        .fetch_optional(pool)
        .await?;
    Ok(record)
}

pub async fn add_to_history(pool: &SqlitePool, cid: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO history (cid) VALUES (?)"
    )
        .bind(cid)
        .execute(pool)
        .await
        .context("FAILED inserting data into history")?;
    Ok(())
}