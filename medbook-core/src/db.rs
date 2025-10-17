use anyhow::Result;
use std::time::Duration;
use tracing::info;

/// Use diesel_async for pooled async connection, instead of
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};

// Use async PG connection instead of
// pub type PgPoolSquad = Pool<ConnectionManager<PgConnection>>
pub type DbPool = Pool<AsyncPgConnection>;

pub async fn connect(database_url: &str) -> Result<DbPool> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .connection_timeout(Duration::from_secs(1))
        .build(config)
        .await;
    info!("Connected to database");
    Ok(pool?)
}
