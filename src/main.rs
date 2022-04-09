mod env;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rusqlite::{Connection, Result};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user));

    let env = env::EnvVars::load();

    test_sqlite().unwrap();

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

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
}

fn test_sqlite() -> Result<Person> {
    let conn = Connection::open("portfolio_tracker.db")?;

    /*     conn.execute(
        "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL
        )",
        [],
    )?; */
    let me = Person {
        id: 0,
        name: "Steven".to_string(),
    };
    conn.execute("INSERT INTO person (name) VALUES (?1)", [&me.name])?;

    let mut stmt = conn.prepare("SELECT id, name FROM person")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    Ok(me)
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    Ok(Json(user))
}

// the input to our `create_user` handler
#[derive(serde::Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(serde::Serialize)]
struct User {
    id: u64,
    username: String,
}
