use std::sync::Arc;

use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_core::{app_state::AppState, outbox};
use medbook_events::{
    OrderCancelSuccessEvent, OrderCancelledEvent, OrderRejectedEvent, OrderRequestedEvent,
    OrderReservedEvent,
};

use crate::schema::inventory;

pub fn reserve_order(delivery: Delivery, state: Arc<AppState>) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderRequestedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;

        // Step 1: Try to reserve stock in one atomic transaction
        let result = conn
            .transaction(move |conn| {
                Box::pin(async move {
                    for item in &payload.order_items {
                        let affected_rows = diesel::update(
                            inventory::table.filter(inventory::product_id.eq(item.product_id)),
                        )
                        .filter(
                            (inventory::total_quantity
                                - inventory::reserved_quantity
                                - inventory::sold_quantity)
                                .ge(item.quantity),
                        )
                        .set(
                            inventory::reserved_quantity
                                .eq(inventory::reserved_quantity + item.quantity),
                        )
                        .execute(conn)
                        .await?;

                        if affected_rows == 0 {
                            return Err(anyhow::anyhow!(
                                "Insufficient stock for product {}",
                                item.product_id
                            ));
                        }
                    }

                    // All items available → insert success outbox
                    outbox::publish(
                        conn,
                        "orders.order_reserved".into(),
                        OrderReservedEvent {
                            order_id: payload.order_id,
                        },
                    )
                    .await?;

                    Ok::<_, anyhow::Error>(())
                })
            })
            .await;

        // Step 2: Handle transaction outcome
        match result {
            Ok(_) => {
                tracing::info!("Reservation for order #{} successful", payload.order_id);
            }
            Err(e) => {
                tracing::error!("Reservation failed: {:?}", e);

                // Independent transaction for "order_rejected" outbox
                let conn = &mut state.db_pool.get().await?;
                outbox::publish(
                    conn,
                    "orders.order_rejected".into(),
                    OrderRejectedEvent {
                        order_id: payload.order_id,
                    },
                )
                .await?;
                tracing::info!("Order #{} rejected, outbox event created", payload.order_id);
            }
        }

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}

pub fn cancel_order(delivery: Delivery, state: Arc<AppState>) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderCancelledEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;

        // Step 1: Try to release reserved stock atomically
        let result = conn
            .transaction(move |conn| {
                Box::pin(async move {
                    for item in &payload.order_items {
                        // Release reserved stock
                        let affected_rows = diesel::update(
                            inventory::table.filter(inventory::product_id.eq(item.product_id)),
                        )
                        // Only release if reserved_quantity >= amount being released
                        .filter(inventory::reserved_quantity.ge(item.quantity))
                        .set(
                            inventory::reserved_quantity
                                .eq(inventory::reserved_quantity - item.quantity),
                        )
                        .execute(conn)
                        .await?;

                        if affected_rows == 0 {
                            return Err(anyhow::anyhow!(
                                "No reserved stock to release for product {}",
                                item.product_id
                            ));
                        }
                    }

                    // Step 2: Publish success outbox event (reservation released)
                    outbox::publish(
                        conn,
                        "orders.order_cancelled".into(),
                        OrderCancelSuccessEvent {
                            order_id: payload.order_id,
                        },
                    )
                    .await?;

                    Ok::<_, anyhow::Error>(())
                })
            })
            .await;

        // Step 3: Handle transaction outcome
        match result {
            Ok(_) => {
                tracing::info!(
                    "Released reserved stock for cancelled order #{}",
                    payload.order_id
                );
            }
            Err(e) => {
                tracing::error!("Failed to release reserved stock: {:?}", e);

                // Optional: publish a compensation event
                let conn = &mut state.db_pool.get().await?;
                outbox::publish(
                    conn,
                    "inventory.reservation_release_failed".into(),
                    OrderRejectedEvent {
                        order_id: payload.order_id,
                    },
                )
                .await?;
            }
        }

        delivery.ack(BasicAckOptions::default()).await?;
        Ok(())
    })
}
