use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;

pub async fn prepare_db_and_get_connection() -> Result<Arc<SqlitePool>> {
    let pool = SqlitePool::connect("porfolio-tracker.db").await?;
    Ok(Arc::new(pool))
}
