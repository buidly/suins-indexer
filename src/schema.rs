// @generated automatically by Diesel CLI.

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

diesel::allow_tables_to_appear_in_same_query!(
    offer_cancelled,
    offer_placed,
);
