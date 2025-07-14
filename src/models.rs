use crate::schema::*;
use diesel::internal::derives::multiconnection::chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use sui_indexer_alt_framework::FieldCount;

#[derive(Insertable, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_placed)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferPlaced {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Insertable, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_cancelled)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferCancelled {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Insertable, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_accepted)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferAccepted {
    pub domain_name: String,
    pub address: String,
    pub owner: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Insertable, Debug, FieldCount, Clone)]
#[diesel(table_name = offer_declined)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OfferDeclined {
    pub domain_name: String,
    pub address: String,
    pub owner: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Insertable, Debug, FieldCount, Clone)]
#[diesel(table_name = make_counter_offer)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MakeCounterOffer {
    pub domain_name: String,
    pub address: String,
    pub owner: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Insertable, Debug, FieldCount, Clone)]
#[diesel(table_name = accept_counter_offer)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AcceptCounterOffer {
    pub domain_name: String,
    pub address: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub tx_digest: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = offers)]
pub struct Offer {
    pub id: Option<i32>,
    pub domain_name: String,
    pub buyer: String,
    pub initial_value: String,
    pub value: String,
    pub owner: Option<String>,
    pub status: OfferStatus,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_tx_digest: String,
}

#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = offers)]
pub struct UpdateOffer {
    pub value: String,
    pub owner: Option<Option<String>>,
    pub status: OfferStatus,
    pub updated_at: DateTime<Utc>,
    pub last_tx_digest: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, AsExpression, FromSqlRow, Serialize, Deserialize,
)]
#[diesel(sql_type = crate::schema::sql_types::Offerstatus)]
pub enum OfferStatus {
    Placed,
    Cancelled,
    Accepted,
    Declined,
    Countered,
    AcceptedCountered,
}

impl diesel::serialize::ToSql<sql_types::Offerstatus, diesel::pg::Pg>
    for OfferStatus
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        let value = match self {
            OfferStatus::Placed => "placed",
            OfferStatus::Cancelled => "cancelled",
            OfferStatus::Accepted => "accepted",
            OfferStatus::Declined => "declined",
            OfferStatus::Countered => "countered",
            OfferStatus::AcceptedCountered => "accepted-countered",
        };
        <str as diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg>>::to_sql(
            value,
            &mut out.reborrow(),
        )
    }
}

impl diesel::deserialize::FromSql<sql_types::Offerstatus, diesel::pg::Pg>
    for OfferStatus
{
    fn from_sql(
        bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let value = <String as diesel::deserialize::FromSql<
            diesel::sql_types::Text,
            diesel::pg::Pg,
        >>::from_sql(bytes)?;
        match value.as_str() {
            "placed" => Ok(OfferStatus::Placed),
            "cancelled" => Ok(OfferStatus::Cancelled),
            "accepted" => Ok(OfferStatus::Accepted),
            "declined" => Ok(OfferStatus::Declined),
            "countered" => Ok(OfferStatus::Countered),
            "accepted-countered" => Ok(OfferStatus::AcceptedCountered),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}
