use anyhow::Result;
use futures::future::BoxFuture;
use futures_lite::StreamExt;
use lapin::message::Delivery;
use std::{sync::Arc, time::Duration};
use tracing::{error, info};

use crate::app_state::AppState;

pub type ConsumerFn = fn(Delivery, Arc<AppState>) -> BoxFuture<'static, Result<()>>;

pub fn init(queue_name: String, consumer_fn: ConsumerFn, state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            let state = state.clone();
            let queue_name = &queue_name;

            let future = Box::pin(async move {
                let channel = state.rmq_client.create_channel().await?;
                channel.create_queue(queue_name).await?;
                let mut consumer = channel.create_consumer(queue_name, queue_name).await?;

                info!("Consumer {} created", queue_name);

                while let Some(delivery) = consumer.next().await {
                    let delivery = delivery?;
                    match consumer_fn(delivery, state.clone()).await {
                        Ok(_) => {}
                        Err(err) => error!("Error in consumer: {}", err),
                    }
                }

                Ok::<_, anyhow::Error>(())
            });

            match future.await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error occured in consumer \"{}\": {:?}", queue_name, e);
                    tracing::error!("Retrying in 5 seconds...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}
