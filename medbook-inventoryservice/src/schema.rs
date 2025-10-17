// @generated automatically by Diesel CLI.

diesel::table! {
    inventory (product_id) {
        product_id -> Int4,
        total_quantity -> Int4,
        reserved_quantity -> Int4,
        sold_quantity -> Int4,
    }
}

diesel::table! {
    outbox (id) {
        id -> Int4,
        event_type -> Text,
        payload -> Text,
        status -> Text,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        th_name -> Text,
        en_name -> Text,
        unit_price -> Float4,
    }
}

diesel::joinable!(inventory -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(inventory, outbox, products,);
