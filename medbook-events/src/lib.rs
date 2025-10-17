use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
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
pub struct OrderPayRequestEvent {
    pub payment_id: Uuid,
    pub order_id: i32,
    pub amount: f32,
    pub provider: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderPaymentSuccessEvent {
    pub payment_id: Uuid,
    pub order_id: i32,
    pub amount: f32,
    pub provider: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeliveryOrderSuccessEvent {
    pub order_id: i32,
}
