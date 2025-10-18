// @generated automatically by Diesel CLI.

diesel::table! {
    deliveries (id) {
        id -> Uuid,
        delivery_address -> Jsonb,
        order_id -> Int4,
        #[max_length = 64]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    delivery_addresses (id) {
        id -> Int4,
        patient_id -> Int4,
        #[max_length = 100]
        recipient_name -> Nullable<Varchar>,
        #[max_length = 20]
        phone_number -> Nullable<Varchar>,
        #[max_length = 255]
        street_address -> Varchar,
        #[max_length = 100]
        city -> Varchar,
        #[max_length = 100]
        state -> Nullable<Varchar>,
        #[max_length = 20]
        postal_code -> Nullable<Varchar>,
        #[max_length = 100]
        country -> Nullable<Varchar>,
        is_default -> Nullable<Bool>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    delivery_logs (id) {
        id -> Uuid,
        delivery_id -> Uuid,
        description -> Text,
        #[max_length = 64]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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

diesel::joinable!(delivery_logs -> deliveries (delivery_id));

diesel::allow_tables_to_appear_in_same_query!(
    deliveries,
    delivery_addresses,
    delivery_logs,
    outbox,
);
