use anyhow::{Context, Result};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use diesel::{ExpressionMethods, QueryDsl, QueryResult, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use medbook_core::app_error::StdResponse;
use medbook_core::{
    aliases::DieselError,
    app_error::AppError,
    app_state::AppState,
    middleware::{self},
    outbox,
};
use medbook_events::OrderCancelledEvent;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    api::get_product_unit_prices,
    models::{CartItemEntity, CreateOrderEntity, OrderEntity},
    schema::{
        cart_items::{self},
        orders::{self},
    },
};

/// Defines all patient-facing order routes (CRUD operations + authorization).
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/patients/orders",
        Router::new()
            .route("/", routing::get(get_orders))
            .route("/", routing::post(create_order))
            .route("/my-orders", routing::get(get_my_orders))
            .route("/{id}", routing::get(get_order))
            .route("/{id}", routing::delete(cancel_order))
            .route_layer(axum::middleware::from_fn(
                middleware::patients_authorization,
            )),
    )
}

/// Fetch all active (non-deleted) orders in the system.
async fn get_orders(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let orders: Vec<OrderEntity> = orders::table
        // .filter(orders::deleted_at.is_null())
        .get_results(conn)
        .await
        .context("Failed to get orders")?;

    Ok(StdResponse {
        data: Some(orders),
        message: Some("Get orders succesfully"),
    })
}

/// Fetch a specific order belonging to the authenticated patient.
#[derive(Serialize)]
struct GetOrderRes {
    pub order: OrderEntity,
    pub order_items: Vec<CartItemEntity>,
    pub total_price: f32,
}

async fn get_order(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let order: QueryResult<OrderEntity> = orders::table
        .find(id)
        // .filter(orders::deleted_at.is_null())
        .filter(orders::patient_id.eq(patient_id))
        .get_result(conn)
        .await;

    if let Err(err) = order {
        match err {
            DieselError::NotFound => return Err(AppError::NotFound),
            _ => return Err(AppError::Other(err.into())),
        }
    }

    let order = order.unwrap();
    let order_items: Vec<CartItemEntity> = cart_items::table
        .filter(cart_items::cart_id.eq(order.cart_id))
        .get_results(conn)
        .await
        .context("Failed to get order items")?;

    let cart_item_ids = order_items.iter().map(|item| item.product_id).collect();
    let unit_prices = get_product_unit_prices(state.http_client, cart_item_ids).await?;
    let total_price: f32 = order_items
        .iter()
        .map(|item| unit_prices.get(&item.product_id).copied().unwrap_or(0.0))
        .sum();

    Ok(StdResponse {
        data: Some(GetOrderRes {
            order,
            order_items,
            total_price,
        }),
        message: Some("Get order successfully"),
    })
}

/// Fetch all orders belonging to the authenticated patient.
async fn get_my_orders(
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let orders: Vec<OrderEntity> = orders::table
        // .filter(orders::deleted_at.is_null())
        .filter(orders::patient_id.eq(patient_id))
        .get_results(conn)
        .await
        .context("Failed to get my orders")?;

    let cart_ids: Vec<i32> = orders.iter().map(|order| order.cart_id).collect();
    let order_items: Vec<CartItemEntity> = cart_items::table
        .filter(cart_items::cart_id.eq_any(&cart_ids))
        .get_results(conn)
        .await
        .context("Failed to get cart items")?;

    let cart_item_ids = order_items.iter().map(|item| item.product_id).collect();
    let unit_prices = get_product_unit_prices(state.http_client, cart_item_ids).await?;

    let mut group: HashMap<i32, Vec<CartItemEntity>> = HashMap::new();
    for item in order_items {
        group.entry(item.cart_id).or_default().push(item);
    }

    let order_with_items: Vec<GetOrderRes> = orders
        .into_iter()
        .map(|order| {
            let order_items = group.remove(&order.cart_id).unwrap_or_default();
            let total_price: f32 = order_items
                .iter()
                .map(|item| unit_prices.get(&item.product_id).copied().unwrap_or(0.0))
                .sum();
            GetOrderRes {
                order_items,
                order,
                total_price,
            }
        })
        .collect();

    Ok(StdResponse {
        data: Some(order_with_items),
        message: Some("Get my orders successfully"),
    })
}

/// Create a new order for the patient and publish an outbox event for inventory reservation.
#[derive(Deserialize)]
struct CreateOrderReq {
    cart_id: i32,
}

async fn create_order(
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
    Json(body): Json<CreateOrderReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let order = conn
        .transaction(move |conn| {
            Box::pin(async move {
                let order = diesel::insert_into(orders::table)
                    .values(CreateOrderEntity {
                        patient_id,
                        cart_id: body.cart_id,
                        status: "PENDING".into(),
                    })
                    .returning(OrderEntity::as_returning())
                    .get_result(conn)
                    .await
                    .context("Failed to create order")?;

                let order_items: Vec<CartItemEntity> = cart_items::table
                    .filter(cart_items::cart_id.eq(order.cart_id))
                    .get_results(conn)
                    .await
                    .context("Failed to get cart items")?;

                let order_items = order_items
                    .iter()
                    .map(|item| medbook_events::OrderItem {
                        product_id: item.product_id,
                        quantity: item.quantity,
                    })
                    .collect();

                outbox::publish(
                    conn,
                    "inventory.reserve_order".into(),
                    medbook_events::OrderRequestedEvent {
                        order_id: order.id,
                        order_items,
                    },
                )
                .await?;

                Ok::<OrderEntity, anyhow::Error>(order)
            })
        })
        .await
        .context("Transaction failed")?;

    Ok(StdResponse {
        data: Some(order),
        message: Some("Create order succesfully"),
    })
}

async fn cancel_order(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let cancelled_order = conn
        .transaction(move |conn| {
            Box::pin(async move {
                let cancelled_order: OrderEntity = diesel::update(orders::table.find(id))
                    .filter(orders::deleted_at.is_null())
                    .filter(orders::patient_id.eq(patient_id))
                    .filter(orders::status.eq("RESERVED"))
                    .set((
                        orders::deleted_at.eq(diesel::dsl::now),
                        orders::status.eq("CANCEL_PENDING"),
                    ))
                    .returning(OrderEntity::as_returning())
                    .get_result(conn)
                    .await
                    .map_err(|_| AppError::NotFound)?;

                let order_items: Vec<CartItemEntity> = cart_items::table
                    .filter(cart_items::cart_id.eq(cancelled_order.cart_id))
                    .get_results(conn)
                    .await
                    .context("Failed to get cart items")?;

                let order_items = order_items
                    .iter()
                    .map(|item| medbook_events::OrderItem {
                        product_id: item.product_id,
                        quantity: item.quantity,
                    })
                    .collect();

                outbox::publish(
                    conn,
                    "inventory.cancel_order".into(),
                    OrderCancelledEvent {
                        order_id: cancelled_order.id,
                        order_items,
                    },
                )
                .await?;

                Ok::<OrderEntity, AppError>(cancelled_order)
            })
        })
        .await?;

    Ok(StdResponse {
        data: Some(cancelled_order),
        message: Some("Cancelled order successfully"),
    })
}
