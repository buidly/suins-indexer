use anyhow::Result;
use sui_indexer_alt_framework::pipeline::concurrent::Handler;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_types::event::Event;
use chrono;
use async_trait::async_trait;
use sui_pg_db::{Connection, Db};
use sui_types::full_checkpoint_content::CheckpointData;
use std::sync::Arc;
use tracing::{error, info};
use sui_indexer_alt_framework::FieldCount;
use bcs::from_bytes;
use diesel_async::RunQueryDsl;

use crate::models::{EventsCursor, OfferCancelled, OfferPlaced};
use crate::schema::{events_cursor, offer_cancelled, offer_placed};

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

pub struct OfferHandler {
    contract_package_id: String,
}

impl OfferHandler {
    pub fn new(contract_package_id: String) -> Self {
        Self {
            contract_package_id,
        }
    }

    fn convert_domain_name(domain_name: &[u8]) -> String {
        String::from_utf8_lossy(domain_name).to_string()
    }

    fn process_event(&self, event: &Event, tx_digest: &str) -> Result<Option<(OfferPlaced, OfferCancelled)>> {
        let event_type = event.type_.to_string();
        if event_type.contains(&self.contract_package_id) {
            if event_type.ends_with("::OfferPlacedEvent") {
                let offer_event: OfferPlacedEvent = from_bytes(&event.contents)?;
                
                let offer = OfferPlaced {
                    domain_name: Self::convert_domain_name(&offer_event.domain_name),
                    address: offer_event.address.to_string(),
                    value: offer_event.value.to_string(),
                    created_at: chrono::Utc::now(),
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some((offer, OfferCancelled {
                    domain_name: String::new(),
                    address: String::new(),
                    value: String::new(),
                    created_at: chrono::Utc::now(),
                    tx_digest: String::new(),
                })));
            } else if event_type.ends_with("::OfferCancelledEvent") {
                info!("Processing OfferCancelledEvent with {} bytes of data", event.contents.len());
                info!("Event contents: {:?}", event.contents);
                
                match from_bytes::<OfferPlacedEvent>(&event.contents) {
                    Ok(test_event) => {
                        info!("Successfully deserialized as OfferPlacedEvent: domain={}, address={}, value={}", 
                              String::from_utf8_lossy(&test_event.domain_name), test_event.address, test_event.value);
                    },
                    Err(e) => {
                        error!("Failed to deserialize as OfferPlacedEvent: {}", e);
                    }
                }
                
                let cancel_event: OfferCancelledEvent = from_bytes(&event.contents)?;
                
                let cancellation = OfferCancelled {
                    domain_name: Self::convert_domain_name(&cancel_event.domain_name),
                    address: cancel_event.address.to_string(),
                    value: cancel_event.value.to_string(),
                    created_at: chrono::Utc::now(),
                    tx_digest: tx_digest.to_string(),
                };

                return Ok(Some((OfferPlaced {
                    domain_name: String::new(),
                    address: String::new(),
                    value: String::new(),
                    created_at: chrono::Utc::now(),
                    tx_digest: String::new(),
                }, cancellation)));
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl Handler for OfferHandler {
    type Store = Db;

    async fn commit<'a>(
        values: &[Self::Value],
        conn: &mut Connection<'a>,
    ) -> anyhow::Result<usize> {
        let mut changes = 0usize;

        info!("Starting commit with {} value batches", values.len());

        for (i, value) in values.iter().enumerate() {
            info!("Processing batch {}: {} offers, {} cancellations", i, value.offers.len(), value.cancellations.len());
            
            // Collect all event cursors 
            let event_cursors: Vec<EventsCursor> = 
                value.offers.iter().map(|e| &e.tx_digest)
                .chain(value.cancellations.iter().map(|e| &e.tx_digest))
                .map(|tx_digest| EventsCursor::from_event(value.checkpoint, tx_digest))
                .collect();

            info!("Created {} event cursors", event_cursors.len());

            if !event_cursors.is_empty() {
                info!("Inserting {} event cursors", event_cursors.len());
                match diesel::insert_into(events_cursor::table)
                    .values(&event_cursors)
                    .execute(conn)
                    .await
                {
                    Ok(count) => info!("Successfully inserted {} event cursors", count),
                    Err(e) => error!("Failed to insert event cursors: {}", e),
                }
            }

            // Store offers
            if !value.offers.is_empty() {
                info!("Inserting {} offers", value.offers.len());
                for (j, offer) in value.offers.iter().enumerate() {
                    info!("Offer {}: domain={}, address={}, value={}", j, offer.domain_name, offer.address, offer.value);
                }
                match diesel::insert_into(offer_placed::table)
                    .values(&value.offers)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} offers", count);
                        changes += count;
                    },
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
                    info!("Cancellation {}: domain={}, address={}, value={}", j, cancellation.domain_name, cancellation.address, cancellation.value);
                }
                match diesel::insert_into(offer_cancelled::table)
                    .values(&value.cancellations)
                    .execute(conn)
                    .await
                {
                    Ok(count) => {
                        info!("Successfully inserted {} cancellations", count);
                        changes += count;
                    },
                    Err(e) => {
                        error!("Failed to insert cancellations: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }

        info!("Commit completed with {} total changes", changes);
        Ok(changes)
    }
}

impl Processor for OfferHandler {
    const NAME: &'static str = "Offer";
    type Value = OfferHandlerValue;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> anyhow::Result<Vec<Self::Value>> {
        info!("Starting process method for checkpoint {}", checkpoint.checkpoint_summary.sequence_number);
        
        let mut offers = Vec::new();
        let mut cancellations = Vec::new();

        info!(
            "Processing checkpoint {} with {} transactions",
            checkpoint.checkpoint_summary.sequence_number,
            checkpoint.transactions.len()
        );
 
        for tx in &checkpoint.transactions {
            let tx_digest = tx.transaction.digest().to_string();
            if let Some(events) = &tx.events {
                for event in &events.data {
                    if event.type_.to_string().ends_with("::OfferPlacedEvent") || event.type_.to_string().ends_with("::OfferCancelledEvent") {
                        info!(
                            "Event type: {} ",
                            event.type_.to_string(),
                        );
                    }
                    match self.process_event(event, &tx_digest) {
                        Ok(Some((offer, cancellation))) => {
                            if !offer.domain_name.is_empty() {
                                info!("Processing offer for domain: {}", offer.domain_name);
                                offers.push(offer);
                            }
                            if !cancellation.domain_name.is_empty() {
                                info!("Processing cancelled offer for domain: {}", cancellation.domain_name);
                                cancellations.push(cancellation);
                            }
                        },
                        Ok(None) => {
                            // No event to process
                        },
                        Err(e) => {
                            error!("Error processing event: {}", e);
                            return Err(e);
                        }
                    }
                }
            }
        }

        info!(
            "Processed checkpoint {}: {} offers, {} cancellations",
            checkpoint.checkpoint_summary.sequence_number,
            offers.len(),
            cancellations.len()
        );

        let result = vec![OfferHandlerValue {
            offers,
            cancellations,
            checkpoint: checkpoint.checkpoint_summary.sequence_number,
        }];
        
        info!("Returning {} handler values from process method", result.len());
        for (i, value) in result.iter().enumerate() {
            info!("Handler value {}: {} offers, {} cancellations, checkpoint {}", 
                  i, value.offers.len(), value.cancellations.len(), value.checkpoint);
        }

        info!("Process method completed successfully");
        Ok(result)
    }
}

pub async fn handle_offer_placed(
    conn: &mut diesel_async::AsyncPgConnection,
    event: &OfferPlacedEvent,
    tx_digest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let new_offer = OfferPlaced {
        domain_name: String::from_utf8_lossy(&event.domain_name).to_string(),
        address: event.address.to_string(),
        value: event.value.to_string(),
        created_at: chrono::Utc::now(),
        tx_digest: tx_digest.to_string(),
    };

    match diesel::insert_into(offer_placed::table)
        .values(&new_offer)
        .execute(conn)
        .await
    {
        Ok(_) => {
            info!("Successfully inserted offer placed for domain: {}", new_offer.domain_name);
            Ok(())
        }
        Err(e) => {
            error!("Error inserting offer placed: {}", e);
            Err(Box::new(e))
        }
    }
}

pub async fn handle_offer_cancelled(
    conn: &mut diesel_async::AsyncPgConnection,
    event: &OfferCancelledEvent,
    tx_digest: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cancelled_offer = OfferCancelled {
        domain_name: String::from_utf8_lossy(&event.domain_name).to_string(),
        address: event.address.to_string(),
        value: event.value.to_string(),
        created_at: chrono::Utc::now(),
        tx_digest: tx_digest.to_string(),
    };

    match diesel::insert_into(offer_cancelled::table)
        .values(&cancelled_offer)
        .execute(conn)
        .await
    {
        Ok(_) => {
            info!("Successfully inserted offer cancelled for domain: {}", cancelled_offer.domain_name);
            Ok(())
        }
        Err(e) => {
            error!("Error inserting offer cancelled: {}", e);
            Err(Box::new(e))
        }
    }
} 