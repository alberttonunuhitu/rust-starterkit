use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use tracing::info;

use crate::config::AppConfig;

pub async fn establish_connection(config: &AppConfig) -> Result<DatabaseConnection, DbErr> {
    let db_config = &config.database;

    let mut opt = ConnectOptions::new(&db_config.url);

    opt.max_connections(db_config.max_connections)
        .min_connections(db_config.min_connections)
        .connect_timeout(Duration::from_secs(db_config.connect_timeout))
        .acquire_timeout(Duration::from_secs(db_config.acquire_timeout))
        .idle_timeout(Duration::from_secs(db_config.idle_timeout))
        .max_lifetime(Duration::from_secs(db_config.max_lifetime))
        .sqlx_logging(false)
        .set_schema_search_path("public");

    info!(
        max_connections = db_config.max_connections,
        "Connecting to PostgreSQL"
    );

    let db = Database::connect(opt).await?;

    db.ping().await.map_err(|e| {
        tracing::error!("Database ping failed: {e}");
        e
    })?;

    info!("PostgreSQL connected successfully");

    Ok(db)
}
