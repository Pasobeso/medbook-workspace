use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{AsChangeset, Identifiable, Insertable, Queryable},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::delivery_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeliveryAddressEntity {
    pub id: i32,
    pub patient_id: i32,
    pub recipient_name: Option<String>,
    pub phone_number: Option<String>,
    pub street_address: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_default: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::deliveries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeliveryEntity {
    pub id: Uuid,
    pub delivery_address: Value,
    pub order_id: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::deliveries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateDeliveryEntity {
    pub delivery_address: Value,
    pub order_id: i32,
    pub status: String,
}

#[derive(Insertable, Serialize, Deserialize, Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::delivery_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateDeliveryAddressEntity {
    pub patient_id: i32,
    pub recipient_name: String,
    pub phone_number: String,
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub is_default: bool,
}

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::delivery_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeliveryLogEntity {
    pub id: Uuid,
    pub delivery_id: Uuid,
    pub description: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::delivery_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateDeliveryLogEntity {
    pub delivery_id: Uuid,
    pub description: String,
    pub status: String,
}
