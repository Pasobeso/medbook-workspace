use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{Identifiable, Insertable, Queryable},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// Carts

#[derive(Queryable, Selectable, Identifiable, Serialize, Debug)]
#[diesel(table_name = crate::schema::carts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CartEntity {
    pub id: i32,
    pub patient_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(belongs_to(CartEntity, foreign_key = cart_id))]
#[diesel(table_name = crate::schema::cart_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CartItemEntity {
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::carts)]
pub struct CreateCartEntity {
    pub patient_id: i32,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::cart_items)]
pub struct CreateCartItemEntity {
    pub cart_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}

// Orders

#[derive(Queryable, Serialize, Selectable, Debug)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderEntity {
    pub id: i32,
    pub cart_id: i32,
    pub patient_id: i32,
    pub status: String,
    pub order_type: String,
    pub delivery_address: Option<Value>,
    pub payment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// #[derive(AsChangeset)]
// #[diesel(table_name = crate::schema::orders)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct UpdateOrderEntity {
//     pub status: Option<String>,
//     pub payment_id: Option<Uuid>,
// }

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOrderEntity {
    pub patient_id: i32,
    pub cart_id: i32,
    pub status: String,
}
