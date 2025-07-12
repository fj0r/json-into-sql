use anyhow::Result;
use r2d2::Pool;
use redis::Client;

pub type RedisPool = Pool<Client>;
pub async fn conn() -> Result<RedisPool> {
    let client = redis::Client::open("redis://127.0.0.1")?;
    let pool = r2d2::Pool::builder().build(client)?;
    Ok(pool)
}
