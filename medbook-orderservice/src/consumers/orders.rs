use std::sync::Arc;

use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_core::app_state::AppState;
use medbook_events::{OrderCancelSuccessEvent, OrderRejectedEvent, OrderReservedEvent};
use tracing::info;

use crate::schema::orders;

pub fn order_reserved(delivery: Delivery, state: Arc<AppState>) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderReservedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        diesel::update(orders::table)
            .filter(orders::id.eq(payload.order_id))
            .set(orders::status.eq("RESERVED"))
            .execute(conn)
            .await?;

        info!("Order #{} has been reserved", payload.order_id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}

pub fn order_rejected(delivery: Delivery, state: Arc<AppState>) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderRejectedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        diesel::update(orders::table)
            .filter(orders::id.eq(payload.order_id))
            .set(orders::status.eq("REJECTED"))
            .execute(conn)
            .await?;

        info!("Order #{} has been rejected", payload.order_id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}

pub fn order_cancel_success(
    delivery: Delivery,
    state: Arc<AppState>,
) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderCancelSuccessEvent =
            serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        diesel::update(orders::table)
            .filter(orders::id.eq(payload.order_id))
            .set(orders::status.eq("CANCELLED"))
            .execute(conn)
            .await?;

        info!("Order #{} has been cancelled", payload.order_id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}

// pub fn order_payment_success(
//     delivery: Delivery,
//     state: Arc<AppState>,
// ) -> BoxFuture<'static, Result<()>> {
//     Box::pin(async move {
//         let conn = &mut state.db_pool.get().await?;
//         let payload: OrderPaymentSuccessEvent =
//             serde_json::from_str(str::from_utf8(&delivery.data)?)?;

//         info!("Received event: {:?}", payload);

//         conn.transaction(move |tx| {
//             Box::pin(async move {
//                 diesel::update(orders::table)
//                     .filter(orders::id.eq(payload.order_id))
//                     .set(orders::status.eq("PAYMENT_SUCCESS"))
//                     .execute(tx)
//                     .await?;

//                 outbox::publish(
//                     tx,
//                     "delivery.order_success".into(),
//                     DeliveryOrderSuccessEvent {
//                         order_id: payload.order_id,
//                     },
//                 )
//                 .await?;

//                 info!("Order #{} has been successfully paid for", 0);

//                 Ok::<_, anyhow::Error>(())
//             })
//         })
//         .await?;

//         delivery.ack(BasicAckOptions::default()).await?;

//         Ok(())
//     })
// }
