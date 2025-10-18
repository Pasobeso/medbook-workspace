use anyhow::Context;
use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use medbook_core::{
    app_error::{AppError, StdResponse},
    app_state::AppState,
};

use crate::{models::DeliveryAddressEntity, schema::delivery_addresses};

/// Defines all patient-facing product routes (CRUD operations + authorization).
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/delivery-addresses",
        Router::new().route("/{id}", routing::get(get_delivery_address)),
    )
}

async fn get_delivery_address(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery_address: DeliveryAddressEntity = delivery_addresses::table
        .find(id)
        .get_result(conn)
        .await
        .map_err(|_| AppError::NotFound)?;

    Ok(StdResponse {
        data: Some(delivery_address),
        message: Some("Get delivery address successfully"),
    })
}
