use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use axum::Router;

use crate::{app_state::AppState, consumers, outbox};

/// Bootstraps a MedBook microservice with common setup steps.
///
/// - Initializes tracing/logging
/// - Loads .env
/// - Creates shared AppState
/// - Starts RabbitMQ consumers
/// - Spawns the outbox worker
/// - Runs the Axum server
pub async fn bootstrap(
    service_name: &str,
    app: Router<AppState>,
    queue_handlers: &[(&str, consumers::ConsumerFn)],
) -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    dotenvy::dotenv().ok();
    info!(".env files loaded");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let ip = format!("0.0.0.0:{}", port);
    info!("Starting {} on {}...", service_name, ip);

    // Shared app state
    let app_state = AppState::init().await?;
    let shared_state = Arc::new(app_state.clone());

    // Start all message consumers
    for (queue_name, handler) in queue_handlers {
        consumers::init(
            queue_name.to_string(),
            handler.clone(),
            shared_state.clone(),
        );
    }

    let app = app.with_state(app_state);

    // Start outbox worker
    outbox::init(shared_state.clone());

    // Start the Axum server
    let listener = TcpListener::bind(&ip).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
