use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, types::chrono};

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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect("postgres://postgres:1234@localhost/postgres").await?;
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    sqlx::migrate!("./dbMigrations")
        .run(&pool)
        .await?;

    assert_eq!(row.0, 150);

    let res = get_lichess().await.expect("Lichess failed");
    
    // https://stackoverflow.com/questions/61561165/how-do-i-define-a-datetime-field-in-sqlx-rust
    // need to enable adding a timestamptz item using chrono
    println!("{res:#?}");
    sqlx::query("INSERT INTO fishnet VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(chrono::DateTime<chrono::Utc>)
        .bind(res.analysis.user.acquired)
        .bind(res.analysis.user.queued)
        .bind(res.analysis.user.oldest)
        .bind(res.analysis.system.acquired)
        .bind(res.analysis.system.queued)
        .bind(res.analysis.system.oldest)
        .execute(&pool)
        .await?;
    Ok(())
    
}

async fn get_lichess() -> Result<Resp, reqwest::Error> {
    let res: Resp = reqwest::get("https://lichess.org/fishnet/status")
        .await?
        .json()
        .await?;

    Ok(res)
}
