use chrono::NaiveDate;
use sqlx::SqlitePool;

pub struct CreateTrade {
    pub ticker: String,
    pub date: String,
    pub r#type: String,
    pub amount: u32,
    pub price: String,
}

pub async fn create_trade(pool: &SqlitePool, trade: CreateTrade) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO trades ( ticker, date, type, amount, price )
        VALUES ( ?1, ?2, ?3, ?4, ?5 )
        "#,
        trade.ticker,
        trade.date,
        trade.r#type,
        trade.amount,
        trade.price
    )
    .execute(pool)
    .await?
    .last_insert_rowid())
}

pub struct ListTrade {
    pub id: i64,
    pub ticker: String,
    pub date: String,
    pub r#type: String,
    pub amount: i64,
    pub price: String,
}

pub async fn list_trades(pool: &SqlitePool) -> Result<Vec<ListTrade>, sqlx::Error> {
    Ok(sqlx::query_as!(
        ListTrade,
        r#"
        SELECT id, ticker, date, type, amount, price FROM trades
        "#,
    )
    .fetch_all(pool)
    .await?)
}

#[derive(Clone)]
pub struct TradeForCalculation {
    pub date: NaiveDate,
    pub amount: i64,
    pub ticker: String,
}

pub async fn list_trades_for_calculation(
    pool: &SqlitePool,
) -> Result<Vec<TradeForCalculation>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT date, amount, ticker FROM trades ORDER BY date asc
        "#,
    )
    .fetch_all(&*pool)
    .await?
    .iter()
    .map(|row| TradeForCalculation {
        amount: row.amount,
        date: NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").unwrap(),
        ticker: row.ticker.clone(),
    })
    .collect())
}

pub async fn delete_trade(pool: &SqlitePool, trade_id: i64) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        DELETE FROM trades WHERE id = ?1
        "#,
        trade_id
    )
    .execute(pool)
    .await?
    .rows_affected())
}
