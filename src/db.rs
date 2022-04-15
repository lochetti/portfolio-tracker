use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct Db(Arc<SqlitePool>);

pub async fn prepare_db_and_get_connection() -> Result<Db> {
    let pool = SqlitePool::connect("porfolio-tracker.db").await?;
    Ok(Db(Arc::new(pool)))
}
