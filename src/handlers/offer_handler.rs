use crate::models::{
    AcceptCounterOffer, MakeCounterOffer, OfferAccepted, OfferCancelled, OfferDeclined, OfferPlaced,
};
use crate::schema::{
    accept_counter_offer, make_counter_offer, offer_accepted, offer_cancelled, offer_declined,
    offer_placed,
};
use anyhow::Context;
use async_trait::async_trait;
use diesel::internal::derives::multiconnection::chrono::{DateTime, Utc};
use diesel_async::RunQueryDsl;
use log::{error, info};
use serde::Deserialize;
use std::sync::Arc;
use sui_indexer_alt_framework::db::{Connection, Db};
use sui_indexer_alt_framework::pipeline::concurrent::Handler;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::types::full_checkpoint_content::CheckpointData;
use sui_indexer_alt_framework::FieldCount;
use sui_indexer_alt_framework::Result;
use sui_types::event::Event;

pub enum OfferEvent {
    Placed(OfferPlaced),
    Cancelled(OfferCancelled),
    Accepted(OfferAccepted),
    Declined(OfferDeclined),
    MakeCounterOffer(MakeCounterOffer),
    AcceptCounterOffer(AcceptCounterOffer),
}

#[derive(serde::Deserialize, Debug)]
pub struct OfferPlacedEvent {
    domain_name: Vec<u8>,
    address: sui_types::base_types::SuiAddress,
    value: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct OfferCancelledEvent {
    domain_name: Vec<u8>,
    address: sui_types::base_types::SuiAddress,
    value: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct OfferAcceptedEvent {
    domain_name: Vec<u8>,
    owner: sui_types::base_types::SuiAddress,
    buyer: sui_types::base_types::SuiAddress,
    value: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct OfferDeclinedEvent {
    domain_name: Vec<u8>,
    owner: sui_types::base_types::SuiAddress,
    buyer: sui_types::base_types::SuiAddress,
    value: u64,
}

// owner can create a counter offer
#[derive(serde::Deserialize, Debug)]
pub struct MakeCounterOfferEvent {
    domain_name: Vec<u8>,
    owner: sui_types::base_types::SuiAddress,
    buyer: sui_types::base_types::SuiAddress,
    value: u64,
}

// buyer can accept counter offer
#[derive(serde::Deserialize, Debug)]
pub struct AcceptCounterOfferEvent {
    domain_name: Vec<u8>,
    buyer: sui_types::base_types::SuiAddress,
    value: u64,
}

#[derive(FieldCount)]
pub struct OfferHandlerValue {
    pub placed: Vec<OfferPlaced>,
    pub cancelled: Vec<OfferCancelled>,
    pub accepted: Vec<OfferAccepted>,
    pub declined: Vec<OfferDeclined>,
    pub make_counter_offer: Vec<MakeCounterOffer>,
    pub accept_counter_offer: Vec<AcceptCounterOffer>,
    pub checkpoint: u64,
}

pub struct OfferHandlerPipeline {
    contract_package_id: String,
}

impl Processor for OfferHandlerPipeline {
    const NAME: &'static str = "Offer";

    type Value = OfferHandlerValue;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let timestamp_ms: u64 = checkpoint.checkpoint_summary.timestamp_ms.into();
        let timestamp_i64 =
            i64::try_from(timestamp_ms).context("Timestamp too large to convert to i64")?;
        let created_at: DateTime<Utc> =
            DateTime::<Utc>::from_timestamp_millis(timestamp_i64).context("invalid timestamp")?;

        let mut placed = Vec::new();
        let mut cancelled = Vec::new();
        let mut accepted = Vec::new();
        let mut declined = Vec::new();
        let mut make_counter_offer = Vec::new();
        let mut accept_counter_offer = Vec::new();

        for tx in &checkpoint.transactions {
            let tx_digest = tx.transaction.digest().to_string();
            if let Some(events) = &tx.events {
                for event in &events.data {
                    match self.process_event(event, &tx_digest, created_at) {
                        Ok(Some(OfferEvent::Placed(offer))) => {
                            info!("Processing placed offer for domain: {}", offer.domain_name);
                            placed.push(offer);
                        }
                        Ok(Some(OfferEvent::Cancelled(offer))) => {
                            info!(
                                "Processing cancelled offer for domain: {}",
                                offer.domain_name
                            );
                            cancelled.push(offer);
                        }
                        Ok(Some(OfferEvent::Accepted(offer))) => {
                            info!(
                                "Processing accepted offer for domain: {}",
                                offer.domain_name
                            );
                            accepted.push(offer);
                        }
                        Ok(Some(OfferEvent::Declined(offer))) => {
                            info!(
                                "Processing declined offer for domain: {}",
                                offer.domain_name
                            );
                            declined.push(offer);
                        }
                        Ok(Some(OfferEvent::MakeCounterOffer(offer))) => {
                            info!(
                                "Processing make counter offer for domain: {}",
                                offer.domain_name
                            );
                            make_counter_offer.push(offer);
                        }
                        Ok(Some(OfferEvent::AcceptCounterOffer(offer))) => {
                            info!(
                                "Processing accept counter offer for domain: {}",
                                offer.domain_name
                            );
                            accept_counter_offer.push(offer);
                        }
                        Ok(None) => {
                            // No event to process
                        }
                        Err(e) => {
                            error!("Error processing event: {}", e);
                            return Err(e);
                        }
                    }
                }
            }
        }

        let result = vec![OfferHandlerValue {
            placed,
            cancelled,
            accepted,
            declined,
            make_counter_offer,
            accept_counter_offer,
            checkpoint: checkpoint.checkpoint_summary.sequence_number,
        }];

        Ok(result)
    }
}

#[async_trait]
impl Handler for OfferHandlerPipeline {
    type Store = Db;

    async fn commit<'a>(values: &[Self::Value], conn: &mut Connection<'a>) -> Result<usize> {
        let mut changes = 0usize;

        for value in values.iter() {
            if !value.placed.is_empty() {
                info!("Inserting {} offers", value.placed.len());

                for (j, offer) in value.placed.iter().enumerate() {
                    info!(
                        "Offer {}: domain={}, address={}, value={}",
                        j, offer.domain_name, offer.address, offer.value
                    );
                }
                match diesel::insert_into(offer_placed::table)
                    .values(&value.placed)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} offers", count);
                        changes += count;
                    }
                    Err(e) => {
                        error!("Failed to insert offers: {}", e);
                        return Err(e.into());
                    }
                }
            }

            if !value.cancelled.is_empty() {
                info!("Inserting {} cancellations", value.cancelled.len());
                for (j, cancellation) in value.cancelled.iter().enumerate() {
                    info!(
                        "Cancellation {}: domain={}, address={}, value={}",
                        j, cancellation.domain_name, cancellation.address, cancellation.value
                    );
                }
                match diesel::insert_into(offer_cancelled::table)
                    .values(&value.cancelled)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} cancellations", count);
                        changes += count;
                    }
                    Err(e) => {
                        error!("Failed to insert cancellations: {}", e);
                        return Err(e.into());
                    }
                }
            }

            if !value.accepted.is_empty() {
                info!("Inserting {} accepted", value.accepted.len());
                for (j, accepted) in value.accepted.iter().enumerate() {
                    info!(
                        "Accepted {}: domain={}, owner={}, address={}, value={}",
                        j, accepted.domain_name, accepted.address, accepted.owner, accepted.value
                    );
                }
                match diesel::insert_into(offer_accepted::table)
                    .values(&value.accepted)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} accepted", count);
                        changes += count;
                    }
                    Err(e) => {
                        error!("Failed to insert accepted: {}", e);
                        return Err(e.into());
                    }
                }
            }

            if !value.declined.is_empty() {
                info!("Inserting {} declined", value.declined.len());
                for (j, declined) in value.declined.iter().enumerate() {
                    info!(
                        "Declined {}: domain={}, owner={}, address={}, value={}",
                        j, declined.domain_name, declined.address, declined.owner, declined.value
                    );
                }
                match diesel::insert_into(offer_declined::table)
                    .values(&value.declined)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} declined", count);
                        changes += count;
                    }
                    Err(e) => {
                        error!("Failed to insert declined: {}", e);
                        return Err(e.into());
                    }
                }
            }

            if !value.make_counter_offer.is_empty() {
                info!(
                    "Inserting {} make counter offer",
                    value.make_counter_offer.len()
                );
                for (j, make_counter_offer) in value.make_counter_offer.iter().enumerate() {
                    info!(
                        "Declined {}: domain={}, owner={}, address={}, value={}",
                        j,
                        make_counter_offer.domain_name,
                        make_counter_offer.address,
                        make_counter_offer.owner,
                        make_counter_offer.value
                    );
                }
                match diesel::insert_into(make_counter_offer::table)
                    .values(&value.make_counter_offer)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} make counter offer", count);
                        changes += count;
                    }
                    Err(e) => {
                        error!("Failed to insert make counter offer: {}", e);
                        return Err(e.into());
                    }
                }
            }

            if !value.accept_counter_offer.is_empty() {
                info!(
                    "Inserting {} accept counter offer",
                    value.accept_counter_offer.len()
                );
                for (j, accept_counter_offer) in value.accept_counter_offer.iter().enumerate() {
                    info!(
                        "Declined {}: domain={}, address={}, value={}",
                        j,
                        accept_counter_offer.domain_name,
                        accept_counter_offer.address,
                        accept_counter_offer.value
                    );
                }
                match diesel::insert_into(accept_counter_offer::table)
                    .values(&value.accept_counter_offer)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} accept counter offer", count);
                        changes += count;
                    }
                    Err(e) => {
                        error!("Failed to insert accept counter offer: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(changes)
    }
}

impl OfferHandlerPipeline {
    pub fn new(contract_package_id: String) -> Self {
        Self {
            contract_package_id,
        }
    }

    fn process_event(
        &self,
        event: &Event,
        tx_digest: &str,
        created_at: DateTime<Utc>,
    ) -> Result<Option<OfferEvent>> {
        let event_type = event.type_.to_string();
        if event_type.starts_with(&self.contract_package_id) {
            info!("Found Auction event: {} ", event_type);

            if event_type.ends_with("::OfferPlacedEvent") {
                let offer_event: OfferPlacedEvent =
                    self.try_deserialize_offer_event(&event.contents)?;
                let offer = OfferPlaced {
                    domain_name: Self::convert_domain_name(&offer_event.domain_name),
                    address: offer_event.address.to_string(),
                    value: offer_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::Placed(offer)));
            } else if event_type.ends_with("::OfferCancelledEvent") {
                let cancel_event: OfferCancelledEvent =
                    self.try_deserialize_offer_event(&event.contents)?;
                let cancellation = OfferCancelled {
                    domain_name: Self::convert_domain_name(&cancel_event.domain_name),
                    address: cancel_event.address.to_string(),
                    value: cancel_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::Cancelled(cancellation)));
            } else if event_type.ends_with("::OfferAcceptedEvent") {
                let accepted_event: OfferAcceptedEvent =
                    self.try_deserialize_offer_event(&event.contents)?;
                let accepted = OfferAccepted {
                    domain_name: Self::convert_domain_name(&accepted_event.domain_name),
                    address: accepted_event.buyer.to_string(),
                    owner: accepted_event.owner.to_string(),
                    value: accepted_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::Accepted(accepted)));
            } else if event_type.ends_with("::OfferDeclinedEvent") {
                let declined_event: OfferDeclinedEvent =
                    self.try_deserialize_offer_event(&event.contents)?;
                let decline = OfferDeclined {
                    domain_name: Self::convert_domain_name(&declined_event.domain_name),
                    address: declined_event.buyer.to_string(),
                    owner: declined_event.owner.to_string(),
                    value: declined_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::Declined(decline)));
            } else if event_type.ends_with("::MakeCounterOfferEvent") {
                let make_counter_offer_event: MakeCounterOfferEvent =
                    self.try_deserialize_offer_event(&event.contents)?;
                let make_counter_offer = MakeCounterOffer {
                    domain_name: Self::convert_domain_name(&make_counter_offer_event.domain_name),
                    address: make_counter_offer_event.buyer.to_string(),
                    owner: make_counter_offer_event.owner.to_string(),
                    value: make_counter_offer_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::MakeCounterOffer(make_counter_offer)));
            } else if event_type.ends_with("::AcceptCounterOfferEvent") {
                let accept_counter_offer_event: AcceptCounterOfferEvent =
                    self.try_deserialize_offer_event(&event.contents)?;
                let accept_counter_offer = AcceptCounterOffer {
                    domain_name: Self::convert_domain_name(&accept_counter_offer_event.domain_name),
                    address: accept_counter_offer_event.buyer.to_string(),
                    value: accept_counter_offer_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::AcceptCounterOffer(accept_counter_offer)));
            }
        }

        Ok(None)
    }

    fn try_deserialize_offer_event<T: for<'a> Deserialize<'a>>(
        &self,
        contents: &[u8],
    ) -> Result<T, anyhow::Error> {
        match bcs::from_bytes::<T>(contents) {
            Ok(event) => Ok(event),
            Err(e) => {
                error!(
                    "Failed to deserialize: {}. Event contents: {:?}",
                    e, contents
                );
                Err(e.into())
            }
        }
    }

    fn convert_domain_name(domain_name: &[u8]) -> String {
        String::from_utf8_lossy(domain_name).to_string()
    }
}
