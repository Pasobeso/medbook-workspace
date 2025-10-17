use anyhow::Result;
use axum::Router;
use medbook_core::bootstrap::bootstrap;
use medbook_orderservice::routes;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .merge(routes::patients::orders::routes())
        .merge(routes::patients::carts::routes());
    bootstrap("OrderService", app, &[]).await?;
    Ok(())
}
