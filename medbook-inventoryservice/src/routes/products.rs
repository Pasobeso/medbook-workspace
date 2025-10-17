use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing,
};

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use medbook_core::{app_error::AppError, app_state::AppState};
use serde::{Deserialize, Serialize};

use crate::{models::ProductEntity, schema::products};

/// Defines all patient-facing product routes (CRUD operations + authorization).
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/products",
        Router::new().route("/", routing::get(get_products)),
    )
}

/// Get products given product_ids
#[derive(Deserialize, Serialize)]
struct GetProductsQuery {
    ids: Option<String>,
}

async fn get_products(
    State(state): State<AppState>,
    Query(query): Query<GetProductsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let ids: Vec<i32> = query
        .ids
        .as_deref()
        .map(|s| {
            s.split(',')
                .filter_map(|id| id.trim().parse::<i32>().ok())
                .collect()
        })
        .unwrap_or_default();

    let products: Vec<ProductEntity>;

    if ids.len() == 0 {
        products = products::table
            .get_results(conn)
            .await
            .context("Failed to get products")?;
    } else {
        products = products::table
            .filter(products::id.eq_any(&ids))
            .get_results(conn)
            .await
            .context("Failed to get products")?;
    }

    Ok(Json(products))
}
