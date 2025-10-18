use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItem {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderRequestedEvent {
    pub order_id: i32,
    pub order_items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderReservedEvent {
    pub order_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderRejectedEvent {
    pub order_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancelledEvent {
    pub order_id: i32,
    pub order_items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderCancelSuccessEvent {
    pub order_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderPayRequestEvent {
    pub payment_id: Uuid,
    pub order_id: i32,
    pub amount: f32,
    pub provider: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeliveryOrderRequestEvent {
    pub order_id: i32,
    pub order_type: String,
    pub delivery_address: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeliveryCreatedEvent {
    pub order_id: i32,
    pub delivery_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeliverySuccessEvent {
    pub order_id: i32,
}
