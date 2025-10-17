use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use diesel::{
    ExpressionMethods, QueryDsl, Selectable, SelectableHelper,
    pg::Pg,
    prelude::{Insertable, Queryable},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;
use tracing::{error, info};

use crate::{app_state::AppState, schema::outbox};

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::outbox)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutboxEntity {
    pub id: i32,
    pub event_type: String,
    pub payload: String,
    pub status: String,
}

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = crate::schema::outbox)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOutboxEntity {
    pub event_type: String,
    pub payload: String,
}

pub fn init(state: Arc<AppState>) {
    info!("Outbox initialized");
    tokio::spawn(async move {
        loop {
            if let Err(e) = start(state.clone()).await {
                error!("Error occured in outbox loop: {:?}", e);
                error!("Retrying in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });
}

async fn start(state: Arc<AppState>) -> anyhow::Result<()> {
    let conn = &mut state.db_pool.get().await?;
    let channel = state.rmq_client.create_channel().await?;

    loop {
        info!("Processing outbox...");

        let events: Vec<OutboxEntity> = outbox::table
            .filter(outbox::status.eq("PENDING"))
            .select(OutboxEntity::as_select())
            .get_results(conn)
            .await?;

        if events.len() == 0 {
            info!("No events to process, sleeping for 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        } else {
            for event in events {
                let queue = channel.create_queue(&event.event_type).await?;
                match queue.publish_plain(&event.payload).await {
                    Ok(_) => {
                        diesel::update(outbox::table.filter(outbox::id.eq(event.id)))
                            .set(outbox::status.eq("PROCESSED"))
                            .execute(conn)
                            .await?;
                        info!(
                            "Outbox event #{} ({}) has been published",
                            event.id, event.event_type
                        )
                    }
                    Err(e) => {
                        tracing::error!(
                            "An error occured while publishing outbox event #{} ({}): {}",
                            event.id,
                            event.event_type,
                            e
                        )
                    }
                };
            }
        }
    }
}

pub async fn publish<C, P>(conn: &mut C, event_type: String, payload: P) -> Result<OutboxEntity>
where
    C: AsyncConnection<Backend = Pg>,
    P: Serialize,
{
    let outbox = diesel::insert_into(outbox::table)
        .values(CreateOutboxEntity {
            event_type: event_type,
            payload: serde_json::to_string(&payload).context("Failed to serialize payload")?,
        })
        .returning(OutboxEntity::as_returning())
        .get_result(conn)
        .await
        .context("Failed to create outbox")?;
    Ok(outbox)
}
