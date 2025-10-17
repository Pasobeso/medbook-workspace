use anyhow::Result;
use reqwest::Client;
use rmq_wrappers::Rmq;

use crate::db::{self, DbPool};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub http_client: Client,
    pub rmq_client: Rmq,
}

impl AppState {
    pub async fn init() -> Result<Self> {
        Ok(Self {
            db_pool: db::connect(&std::env::var("DATABASE_URL")?).await?,
            http_client: Client::new(),
            rmq_client: Rmq::connect(&std::env::var("RMQ_URL")?).await?,
        })
    }
}
