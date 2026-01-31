use sqlx::{PgPool, Pool, Postgres};
use std::env;

pub mod queries;

pub type DatabasePool = Pool<Postgres>;

pub async fn create_pool() -> Result<DatabasePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url).await?;

    Ok(pool)
}

pub async fn health_check(pool: &DatabasePool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
