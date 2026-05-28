use anyhow::Result;
use commom::constants::{
    ENCRYPTION_ALGORITHM, SYSTEM_DATABASE_PATH, SYSTEM_FOREIGN_DATA_PATH, USER_DATA_FOLDER_PATH,
};
use commom::database::Database;
use glifo::credentials::{self, Credentials};
use glifo::encrypter::CryptoAlgorithm;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::Arc;
use tokio::fs;

#[derive(Clone)]
pub struct System {
    pub database: Arc<Database>,
    pub credentials: Arc<Credentials>,
    pub encryption_system: Arc<CryptoAlgorithm>,
}

pub async fn system_startup() -> Result<System> {
    fs::create_dir_all(SYSTEM_FOREIGN_DATA_PATH).await?; // ensures system folders existence
    fs::create_dir_all(USER_DATA_FOLDER_PATH).await?; // Ensures userdata default folder existence
    let pool = init_db().await?;
    let credentials = credentials::handle_credentials().await?;
    let database = Database::build(Some(pool)).await?;
    let encryption_system = match ENCRYPTION_ALGORITHM {
        "aes" => CryptoAlgorithm::AES,
        "chacha" => CryptoAlgorithm::ChaCha20,
        "twofish" => CryptoAlgorithm::Twofish,
        "serpent" => CryptoAlgorithm::Serpent,
        _ => panic!("FATAL: unrecognizable encryption algorithm"),
    };
    Ok(System {
        database: Arc::new(database),
        credentials: Arc::new(credentials),
        encryption_system: Arc::new(encryption_system),
    })
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
            CREATE TABLE IF NOT EXISTS upload_history (
                cid TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL
            );
            "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS perf_points (
                id INTEGER PRIMARY KEY,
                strategy TEXT NOT NULL,
                init_timestamp INTEGER NOT NULL,
                final_timestamp INTEGER NOT NULL,
                operation TEXT NOT NULL,
                payload_size INTEGER NOT NULL
            );
            "#,
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}
