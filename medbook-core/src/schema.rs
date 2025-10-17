// @generated automatically by Diesel CLI.

diesel::table! {
    outbox (id) {
        id -> Int4,
        event_type -> Text,
        payload -> Text,
        status -> Text,
    }
}

diesel::table! {
    payments (id) {
        id -> Uuid,
        order_id -> Int4,
        amount -> Float4,
        #[max_length = 32]
        status -> Varchar,
        #[max_length = 64]
        provider -> Varchar,
        #[max_length = 128]
        provider_ref -> Nullable<Varchar>,
        failure_reason -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(outbox, payments,);
