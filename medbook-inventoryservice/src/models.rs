use diesel::{
    Selectable,
    prelude::{Insertable, Queryable},
};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductEntity {
    pub id: i32,
    pub en_name: String,
    pub th_name: String,
    pub unit_price: f32,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::inventory)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InventoryEntity {
    pub product_id: i32,
    pub total_quantity: i32,
    pub reserved_quantity: i32,
    pub sold_quantity: i32,
}
