use crate::schema::{offer_cancelled, offer_placed};
use diesel::prelude::*;
use sui_indexer_alt_framework::FieldCount;
use diesel::internal::derives::multiconnection::chrono::{DateTime, Utc};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_placed)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferPlaced {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_cancelled)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferCancelled {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}
