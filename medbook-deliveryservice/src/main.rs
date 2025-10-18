use anyhow::Result;
use axum::Router;
use medbook_core::bootstrap::bootstrap;
use medbook_deliveryservice::{consumers, routes};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .merge(routes::deliveries::routes())
        .merge(routes::delivery_addresses::routes())
        .merge(routes::patients::delivery_addresses::routes());

    bootstrap(
        "DeliveryService",
        app,
        &[(
            "delivery.order_request",
            consumers::deliveries::order_request,
        )],
    )
    .await?;
    Ok(())
}
