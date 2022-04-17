mod db;

use anyhow::Result;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use dotenv::dotenv;
use sqlx::SqlitePool;
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
