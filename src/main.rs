use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool, types::chrono};
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


#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect("postgres://postgres:1234@localhost/postgres").await?;
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;
    assert_eq!(row.0, 150);

    sqlx::migrate!("./dbMigrations")
        .run(&pool)
        .await?;
    

    let mut interval = time::interval(time::Duration::from_secs(60));
    for _i in 0..3 {
        interval.tick().await;
        handle_stats(&pool).await?;
    }
    
    
    Ok(())
    
}

async fn handle_stats(db: &PgPool) -> Result<(), sqlx::Error> {

    let res = get_lichess().await.expect("Lichess failed");
    println!("{res:#?}");
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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}