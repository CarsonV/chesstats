use axum::extract::State;
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgPoolOptions,
    types::chrono::{self, Utc},
    PgPool, Row,
};
use tokio::time;


#[derive(Debug, Deserialize)]
struct Inner {
    acquired: i32,
    queued: i32,
    oldest: i32,
}
#[derive(Debug, Deserialize)]
struct Outer {
    user: Inner,
    system: Inner,
}
#[derive(Debug, Deserialize)]
struct Resp {
    analysis: Outer,
}

#[derive(Serialize)]
struct FlattenedResponse {
    time: chrono::DateTime<Utc>,
    user_acquired: i32,
    user_queued: i32,
    user_oldest: i32,
    system_acquired: i32,
    system_queued: i32,
    system_oldest: i32,
}
#[tokio::main]
async fn main() {
    let pool = init_db().await.expect("DB Init failed");
    //different pools for the api and lichess stuff to avoid lifetime issues
    let axum_pool = init_db().await.expect("Axum pool Init failed");

    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            handle_stats(&pool).await.expect("Main task failed");
        }
    });
    //https://github.com/brannan/realworld-axum-sqlx/blob/main/src/http/users.rs
    let app = Router::new()
        .route("/", get(last_user_acquired))
        .route("/last", get(get_last))
        .with_state(axum_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
//TODO! look at having this function take in a set value of number of connections. One for lichess pool and another for api
async fn init_db() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect("postgres://postgres:1234@localhost/postgres")
        .await?;
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await?;
    assert_eq!(row.0, 150);

    sqlx::migrate!("./dbMigrations").run(&pool).await?;

    Ok(pool)
}
//.expect("Lichess failed");
async fn handle_stats(db: &PgPool) -> Result<(), sqlx::Error> {
    let res = match get_lichess().await {
        Ok(data) => data,
        Err(_) => {
            eprintln!("First request failed, trying in 2 minutes");
            time::sleep(time::Duration::from_secs(120)).await;
            get_lichess().await.expect("Lichess failed after retry")
        }
    };
    //println!("{res:#?}"); // used for debug outputting to console, dont need for release
    write_data(&db, &res).await.expect("DB write failed");

    Ok(())
}

async fn get_lichess() -> Result<Resp, reqwest::Error> {
    let res: Resp = reqwest::get("https://lichess.org/fishnet/status")
        .await?
        .json()
        .await?;

    Ok(res)
}
async fn write_data(db: &PgPool, res: &Resp) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO fishnet VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(chrono::Utc::now())
        .bind(res.analysis.user.acquired)
        .bind(res.analysis.user.queued)
        .bind(res.analysis.user.oldest)
        .bind(res.analysis.system.acquired)
        .bind(res.analysis.system.queued)
        .bind(res.analysis.system.oldest)
        .execute(db)
        .await?;
    Ok(())
}
//https://github.com/launchbadge/sqlx/blob/main/examples/postgres/json/src/main.rs

async fn get_last(State(state): State<PgPool>) -> Json<FlattenedResponse> {
    let response = sqlx::query_as!(FlattenedResponse,
        r#"SELECT * FROM FISHNET ORDER BY time  DESC LIMIT 1"#)
        .fetch_one(&state)
        .await
        .expect("Failed full last DB read");

    Json(response)
}


async fn last_user_acquired(State(state): State<PgPool>) -> Json<i32> {
    let row = sqlx::query("SELECT user_acquired FROM fishnet ORDER BY time DESC LIMIT 1")
        .fetch_one(&state)
        .await
        .expect("Failed DB read");

    let data: i32 = row.get("user_acquired");

    println!("{data}");
    Json(data)
}

/*
TODOS

look at query as and return an array or vec of the last x values
cleanup code, look into seperating out the fishnet query code and the api probably
Still need to implement some testing, would make life easier
 */

 //    r#"SELECT time AS "time!", user_acquired, user_queued, user_oldest, system_acquired, system_queued, system_oldest FROM FISHNET ORDER BY time  DESC LIMIT 1"#)

