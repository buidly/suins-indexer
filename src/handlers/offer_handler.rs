use anyhow::Context;
use async_trait::async_trait;
use diesel::internal::derives::multiconnection::chrono::{DateTime, Utc};
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use sui_indexer_alt_framework::db::{Connection, Db};
use sui_indexer_alt_framework::pipeline::concurrent::Handler;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::types::full_checkpoint_content::CheckpointData;
use sui_indexer_alt_framework::FieldCount;
use sui_indexer_alt_framework::Result;
use sui_types::event::Event;
use log::{error, info};

use crate::models::{OfferCancelled, OfferPlaced};
use crate::schema::{offer_cancelled, offer_placed};

pub enum OfferEvent {
    Placed(OfferPlaced),
    Cancelled(OfferCancelled),
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

#[derive(FieldCount)]
pub struct OfferHandlerValue {
    pub offers: Vec<OfferPlaced>,
    pub cancellations: Vec<OfferCancelled>,
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

        let checkpoint_id: i64 = checkpoint.checkpoint_summary.sequence_number.try_into()?;

        let mut offers = Vec::new();
        let mut cancellations = Vec::new();

        for tx in &checkpoint.transactions {
            let tx_digest = tx.transaction.digest().to_string();
            if let Some(events) = &tx.events {
                for event in &events.data {
                    match self.process_event(event, &tx_digest, created_at) {
                        Ok(Some(OfferEvent::Placed(offer))) => {
                            info!("Processing offer for domain: {}", offer.domain_name);
                            offers.push(offer);
                        }
                        Ok(Some(OfferEvent::Cancelled(cancellation))) => {
                            info!(
                                "Processing cancelled offer for domain: {}",
                                cancellation.domain_name
                            );
                            cancellations.push(cancellation);
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
            offers,
            cancellations,
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

        for (i, value) in values.iter().enumerate() {
            // Store offers
            if !value.offers.is_empty() {
                info!("Inserting {} offers", value.offers.len());
                for (j, offer) in value.offers.iter().enumerate() {
                    info!(
                        "Offer {}: domain={}, address={}, value={}",
                        j, offer.domain_name, offer.address, offer.value
                    );
                }
                match diesel::insert_into(offer_placed::table)
                    .values(&value.offers)
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

            // Store cancellations
            if !value.cancellations.is_empty() {
                info!("Inserting {} cancellations", value.cancellations.len());
                for (j, cancellation) in value.cancellations.iter().enumerate() {
                    info!(
                        "Cancellation {}: domain={}, address={}, value={}",
                        j, cancellation.domain_name, cancellation.address, cancellation.value
                    );
                }
                match diesel::insert_into(offer_cancelled::table)
                    .values(&value.cancellations)
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
                let offer_event = self.try_deserialize_offer_placed_event(&event.contents)?;
                let offer = OfferPlaced {
                    domain_name: Self::convert_domain_name(&offer_event.domain_name),
                    address: offer_event.address.to_string(),
                    value: offer_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::Placed(offer)));
            } else if event_type.ends_with("::OfferCancelledEvent") {
                let cancel_event = self.try_deserialize_offer_cancelled_event(&event.contents)?;
                let cancellation = OfferCancelled {
                    domain_name: Self::convert_domain_name(&cancel_event.domain_name),
                    address: cancel_event.address.to_string(),
                    value: cancel_event.value.to_string(),
                    created_at,
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some(OfferEvent::Cancelled(cancellation)));
            }
        }

        Ok(None)
    }

    fn try_deserialize_offer_placed_event(
        &self,
        contents: &[u8],
    ) -> Result<OfferPlacedEvent, anyhow::Error> {
        match bcs::from_bytes::<OfferPlacedEvent>(contents) {
            Ok(event) => Ok(event),
            Err(e) => {
                error!(
                    "Failed to deserialize as OfferPlacedEvent: {}. Event contents: {:?}",
                    e, contents
                );
                Err(e.into())
            }
        }
    }

    fn try_deserialize_offer_cancelled_event(
        &self,
        contents: &[u8],
    ) -> Result<OfferCancelledEvent, anyhow::Error> {
        match bcs::from_bytes::<OfferCancelledEvent>(contents) {
            Ok(event) => Ok(event),
            Err(e) => {
                error!(
                    "Failed to deserialize as OfferPlacedEvent: {}. Event contents: {:?}",
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
