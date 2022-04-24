mod db;

use anyhow::Result;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::format::ParseError;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use dotenv::dotenv;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = match db::prepare_db_and_get_connection().await {
        Ok(pool) => pool,
        Err(e) => {
            println!("Error creating preparing database connection {}", e);
            return;
        }
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/trades", post(create_trade))
        .route("/trades", get(list_trades))
        .route("/trades/:trade_id", delete(delete_trade))
        .route("/prices", get(list_prices))
        .route("/prices/update", get(update_prices))
        .layer(Extension(pool));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(serde::Deserialize)]
struct CreateTrade {
    ticker: String,
    date: String,
    r#type: String,
    amount: u32,
    price: String,
}

async fn create_trade(
    pool: Extension<Arc<SqlitePool>>,
    Json(payload): Json<CreateTrade>,
) -> Result<Json<i64>, StatusCode> {
    let id = match sqlx::query!(
        r#"
        INSERT INTO trades ( ticker, date, type, amount, price )
        VALUES ( ?1, ?2, ?3, ?4, ?5 )
        "#,
        payload.ticker,
        payload.date,
        payload.r#type,
        payload.amount,
        payload.price
    )
    .execute(&*pool.0)
    .await
    {
        Ok(res) => res.last_insert_rowid(),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(id))
}

#[derive(serde::Serialize)]
struct ListTradesResponse {
    id: i64,
    ticker: String,
    date: String,
    r#type: String,
    amount: i64,
    price: String,
}

async fn list_trades(
    pool: Extension<Arc<SqlitePool>>,
) -> Result<Json<Vec<ListTradesResponse>>, StatusCode> {
    let list_of_trades = match sqlx::query_as!(
        ListTradesResponse,
        r#"
        SELECT id, ticker, date, type, amount, price FROM trades
        "#,
    )
    .fetch_all(&*pool.0)
    .await
    {
        Ok(res) => res,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(list_of_trades))
}

async fn delete_trade(Path(trade_id): Path<i64>, pool: Extension<Arc<SqlitePool>>) -> StatusCode {
    let deleted_count = match sqlx::query!(
        r#"
        DELETE FROM trades WHERE id = ?1
        "#,
        trade_id
    )
    .execute(&*pool.0)
    .await
    {
        Ok(res) => res.rows_affected(),
        Err(_e) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if deleted_count == 1 {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Deserialize)]
struct AlphaVantageDailyPriceResponse {
    #[serde(rename(deserialize = "4. close"))]
    price: String,
}

#[derive(Deserialize)]
struct AlphaVantagePriceApiResponse {
    #[serde(rename(deserialize = "Time Series (Daily)"))]
    time_series: HashMap<String, AlphaVantageDailyPriceResponse>,
}

#[derive(serde::Serialize)]
struct LastPriceDate {
    date: String,
}

async fn update_prices(pool: Extension<Arc<SqlitePool>>) -> StatusCode {
    let mut api_output_size = "full";
    let last_ticker_date = sqlx::query_as!(
        LastPriceDate,
        r#"
        SELECT date from prices where ticker = 'IWDA' ORDER BY date desc limit 1
        "#
    )
    .fetch_one(&*pool.0)
    .await
    .unwrap();
    let last_ticker_date = NaiveDate::parse_from_str(&last_ticker_date.date, "%Y-%m-%d").unwrap();

    if last_ticker_date > Utc::today().naive_utc() + Duration::days(-100) {
        api_output_size = "compact";
    }

    let alpha_adavantage_key = match env::var("ALPHA_VANTAGE_API_KEY") {
        Ok(key) => key,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let url = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=IWDA.AMS&apikey={}&outputsize={}", alpha_adavantage_key, api_output_size);
    let resp = reqwest::get(url)
        .await
        .unwrap()
        .json::<AlphaVantagePriceApiResponse>()
        .await
        .unwrap();
    let prices_to_insert = resp.time_series.iter().filter(|price| {
        let date = NaiveDate::parse_from_str(price.0, "%Y-%m-%d").unwrap();
        date > last_ticker_date
    });
    for (key, val) in prices_to_insert {
        sqlx::query!(
            r#"
            INSERT INTO prices ( ticker, date, price )
            VALUES ( ?1, ?2, ?3 )
            "#,
            "IWDA",
            key,
            val.price
        )
        .execute(&*pool.0)
        .await
        .unwrap();
    }
    StatusCode::OK
}

#[derive(serde::Serialize)]
struct ListPricesResponse {
    id: i64,
    ticker: String,
    date: String,
    price: String,
}

async fn list_prices(
    pool: Extension<Arc<SqlitePool>>,
) -> Result<Json<Vec<ListPricesResponse>>, StatusCode> {
    let list_of_prices = match sqlx::query_as!(
        ListPricesResponse,
        r#"
        SELECT id, ticker, date, price FROM prices ORDER by date asc
        "#,
    )
    .fetch_all(&*pool.0)
    .await
    {
        Ok(res) => res,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(list_of_prices))
}
