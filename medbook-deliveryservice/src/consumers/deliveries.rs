use std::sync::Arc;

use anyhow::{Context, Result};
use diesel::SelectableHelper;
use diesel_async::{AsyncConnection, RunQueryDsl};
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_core::{app_error::AppError, app_state::AppState, outbox};
use medbook_events::{DeliveryCreatedEvent, DeliveryOrderRequestEvent};
use tracing::info;

use crate::{
    models::{CreateDeliveryEntity, DeliveryEntity},
    schema::deliveries,
};

pub fn order_request(delivery: Delivery, state: Arc<AppState>) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: DeliveryOrderRequestEvent =
            serde_json::from_str(str::from_utf8(&delivery.data)?)?;

        let deliv = conn
            .transaction(move |conn| {
                Box::pin(async move {
                    let deliv: DeliveryEntity = diesel::insert_into(deliveries::table)
                        .values(CreateDeliveryEntity {
                            delivery_address: payload.delivery_address,
                            order_id: payload.order_id,
                            status: "PREPARING".into(),
                        })
                        .returning(DeliveryEntity::as_returning())
                        .get_result(conn)
                        .await
                        .context("Failed to create delivery")?;

                    outbox::publish(
                        conn,
                        "orders.delivery_created".into(),
                        DeliveryCreatedEvent {
                            order_id: payload.order_id,
                            delivery_id: deliv.id,
                        },
                    )
                    .await
                    .context("Failed to create outbox")?;

                    Ok::<DeliveryEntity, AppError>(deliv)
                })
            })
            .await
            .context("Transaction failed")?;

        info!("Created delivery: {:?}", deliv);
        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}
