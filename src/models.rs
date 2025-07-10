// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::schema::{events_cursor, offer_cancelled, offer_placed};
use diesel::prelude::*;
use sui_indexer_alt_framework::FieldCount;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, FieldCount, Clone)]
#[diesel(table_name = events_cursor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventsCursor {
    pub checkpoint: String,
    pub tx_digest: String,
}

impl EventsCursor {
    pub fn from_event(checkpoint: u64, tx_digest: &str) -> Self {
        Self {
            checkpoint: checkpoint.to_string(),
            tx_digest: tx_digest.to_string(),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_placed)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferPlaced {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tx_digest: String,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_cancelled)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferCancelled {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tx_digest: String,
}
 