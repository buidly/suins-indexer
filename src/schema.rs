// @generated automatically by Diesel CLI.

diesel::table! {
    events_cursor (id) {
        id -> Int4,
        checkpoint -> Varchar,
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
    events_cursor,
    offer_cancelled,
    offer_placed,
    watermarks,
);
