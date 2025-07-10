// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "offerstatus"))]
    pub struct Offerstatus;
}

diesel::table! {
    accept_counter_offer (id) {
        id -> Int4,
        domain_name -> Varchar,
        address -> Varchar,
        value -> Varchar,
        created_at -> Timestamptz,
        tx_digest -> Varchar,
    }
}

diesel::table! {
    make_counter_offer (id) {
        id -> Int4,
        domain_name -> Varchar,
        address -> Varchar,
        owner -> Varchar,
        value -> Varchar,
        created_at -> Timestamptz,
        tx_digest -> Varchar,
    }
}

diesel::table! {
    offer_accepted (id) {
        id -> Int4,
        domain_name -> Varchar,
        address -> Varchar,
        owner -> Varchar,
        value -> Varchar,
        created_at -> Timestamptz,
        tx_digest -> Varchar,
    }
}

diesel::table! {
    offer_cancelled (id) {
        id -> Int4,
        domain_name -> Varchar,
        address -> Varchar,
        value -> Varchar,
        created_at -> Timestamptz,
        tx_digest -> Varchar,
    }
}

diesel::table! {
    offer_declined (id) {
        id -> Int4,
        domain_name -> Varchar,
        address -> Varchar,
        owner -> Varchar,
        value -> Varchar,
        created_at -> Timestamptz,
        tx_digest -> Varchar,
    }
}

diesel::table! {
    offer_placed (id) {
        id -> Int4,
        domain_name -> Varchar,
        address -> Varchar,
        value -> Varchar,
        created_at -> Timestamptz,
        tx_digest -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Offerstatus;

    offers (id) {
        id -> Int4,
        domain_name -> Varchar,
        buyer -> Varchar,
        initial_value -> Varchar,
        value -> Varchar,
        owner -> Nullable<Varchar>,
        status -> Offerstatus,
        updated_at -> Timestamptz,
        created_at -> Timestamptz,
        last_tx_digest -> Varchar,
    }
}

diesel::table! {
    watermarks (pipeline) {
        pipeline -> Text,
        epoch_hi_inclusive -> Int8,
        checkpoint_hi_inclusive -> Int8,
        tx_hi -> Int8,
        timestamp_ms_hi_inclusive -> Int8,
        reader_lo -> Int8,
        pruner_timestamp -> Timestamp,
        pruner_hi -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    accept_counter_offer,
    make_counter_offer,
    offer_accepted,
    offer_cancelled,
    offer_declined,
    offer_placed,
    offers,
    watermarks,
);
