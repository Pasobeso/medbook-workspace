use anyhow::Result;
use axum::Router;
use medbook_core::bootstrap::bootstrap;
use medbook_inventoryservice::routes;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().merge(routes::products::routes());
    bootstrap("InventoryService", app, &[]).await?;
    Ok(())
}
