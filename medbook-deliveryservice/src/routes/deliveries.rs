use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};

use medbook_core::{
    app_error::{AppError, StdResponse},
    app_state::AppState,
    outbox,
};
use medbook_events::DeliverySuccessEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{CreateDeliveryLogEntity, DeliveryEntity, DeliveryLogEntity},
    schema::{deliveries, delivery_logs},
};

/// Defines all patient-facing product routes (CRUD operations + authorization).
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/deliveries",
        Router::new()
            .route("/", routing::get(get_deliveries))
            .route("/{id}", routing::get(get_delivery))
            .route("/{id}/status", routing::patch(update_delivery_state)),
    )
}

#[derive(Serialize)]
struct GetDeliveryRes {
    delivery: DeliveryEntity,
    delivery_logs: Vec<DeliveryLogEntity>,
}

async fn get_delivery(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery: DeliveryEntity = deliveries::table
        .find(id)
        .get_result(conn)
        .await
        .context("Failed to get delivery")?;

    let delivery_logs: Vec<DeliveryLogEntity> = delivery_logs::table
        .filter(delivery_logs::delivery_id.eq(delivery.id))
        .order_by(delivery_logs::updated_at.desc())
        .get_results(conn)
        .await
        .context("Failed to get delivery logs")?;

    Ok(StdResponse {
        data: Some(GetDeliveryRes {
            delivery,
            delivery_logs,
        }),
        message: Some("Get delivery successfully"),
    })
}

async fn get_deliveries(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let deliveries: Vec<DeliveryEntity> = deliveries::table
        .get_results(conn)
        .await
        .context("Failed to get deliveries")?;

    Ok(StdResponse {
        data: Some(deliveries),
        message: Some("Get deliveries successfully"),
    })
}

#[derive(Deserialize)]
struct UpdateDeliveryStateReq {
    status: String,
    description: String,
}

#[derive(Serialize)]
struct UpdateDeliveryStateRes {
    updated_delivery: DeliveryEntity,
    delivery_log: DeliveryLogEntity,
}

async fn update_delivery_state(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<UpdateDeliveryStateReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let allowed_statuses = ["PREPARING", "EN_ROUTE", "DELIVERED"];
    if !allowed_statuses.contains(&body.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Allowed statuses are: {}",
            allowed_statuses.join(", ")
        )));
    }

    let (updated_delivery, delivery_log) = conn
        .transaction(move |conn| {
            Box::pin(async move {
                let delivery: DeliveryEntity = diesel::update(deliveries::table.find(id))
                    .set(deliveries::status.eq(body.status.clone()))
                    .returning(DeliveryEntity::as_returning())
                    .get_result(conn)
                    .await
                    .context("Failed to update delivery status")?;

                let delivery_log = diesel::insert_into(delivery_logs::table)
                    .values(CreateDeliveryLogEntity {
                        delivery_id: delivery.id.clone(),
                        description: body.description,
                        status: body.status,
                    })
                    .returning(DeliveryLogEntity::as_returning())
                    .get_result(conn)
                    .await
                    .context("Failed to create delivery log")?;

                if delivery.status.as_str() == "DELIVERED" {
                    outbox::publish(
                        conn,
                        "orders.delivery_success".into(),
                        DeliverySuccessEvent {
                            order_id: delivery.order_id,
                        },
                    )
                    .await
                    .context("Failed to send outbox")?;
                }

                Ok::<(DeliveryEntity, DeliveryLogEntity), AppError>((delivery, delivery_log))
            })
        })
        .await
        .context("Transaction failed")?;

    Ok(StdResponse {
        data: Some(UpdateDeliveryStateRes {
            updated_delivery,
            delivery_log,
        }),
        message: Some("Get deliveries successfully"),
    })
}
