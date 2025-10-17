// @generated automatically by Diesel CLI.

diesel::table! {
    cart_items (cart_id, product_id) {
        cart_id -> Int4,
        product_id -> Int4,
        quantity -> Int4,
        unit_price -> Float4,
        total_price -> Nullable<Float4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    carts (id) {
        id -> Int4,
        patient_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        cart_id -> Int4,
        patient_id -> Int4,
        status -> Text,
        order_type -> Text,
        delivery_address -> Nullable<Jsonb>,
        payment_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
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

diesel::joinable!(cart_items -> carts (cart_id));

diesel::allow_tables_to_appear_in_same_query!(cart_items, carts, orders, outbox,);
