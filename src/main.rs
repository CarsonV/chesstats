use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;

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
async fn main() -> Result<(), reqwest::Error> {

    let mainRes = get_lichess();
    
    
    println!("{mainRes:#?}");

    Ok(())
    
}

async fn get_lichess() -> Result<Resp, reqwest::Error> {
    let res: Resp = reqwest::get("https://lichess.org/fishnet/status")
        .await?
        .json()
        .await?;

    Ok(res)
}
