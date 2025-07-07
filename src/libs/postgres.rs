use super::config::Database;
use anyhow::Result;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub async fn conn(config: &Database) -> Result<Pool<Postgres>> {
    let c: String = config.to_url();
    let pool = PgPoolOptions::new().max_connections(5).connect(&c).await?;
    Ok(pool)
}
