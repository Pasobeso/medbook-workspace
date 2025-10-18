use anyhow::Result;
use axum::Router;
use medbook_core::bootstrap::bootstrap;
use medbook_inventoryservice::{consumers, routes};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().merge(routes::products::routes());
    bootstrap(
        "InventoryService",
        app,
        &[
            (
                "inventory.reserve_order",
                consumers::inventory::reserve_order,
            ),
            ("inventory.cancel_order", consumers::inventory::cancel_order),
        ],
    )
    .await?;
    Ok(())
}
