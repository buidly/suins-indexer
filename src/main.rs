use anyhow::Context;
use clap::Parser;
use prometheus::Registry;
use reqwest::Url;
use std::net::SocketAddr;
use sui_indexer_alt_framework::db::DbArgs;
use sui_indexer_alt_framework::ingestion::ClientArgs;
use sui_indexer_alt_framework::{Indexer, IndexerArgs};
use sui_indexer_alt_metrics::{MetricsArgs, MetricsService};
use suins_indexer::handlers::offer_handler::OfferHandler;
use suins_indexer::{CONTRACT_PACKAGE_ID, DATABASE_URL, METRICS_ADDRESS, REMOTE_STORE_URL, RPC_API_URL};
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Parser)]
#[clap(rename_all = "kebab-case", author, version)]
struct Args {
    #[command(flatten)]
    db_args: DbArgs,
    #[command(flatten)]
    indexer_args: IndexerArgs,
    #[clap(env, long, default_value = METRICS_ADDRESS)]
    metrics_address: SocketAddr,
    #[clap(env, long, default_value = DATABASE_URL)]
    database_url: Url,
    /// Checkpoint remote store URL, defaulted to Sui testnet remote store.
    #[clap(env, long, default_value = REMOTE_STORE_URL)]
    remote_store_url: Url,
    /// Contract package ID for the offer events
    #[clap(env, long, default_value = CONTRACT_PACKAGE_ID)]
    contract_package_id: String,
    /// RPC URL for the Sui testnet
    #[clap(env, long, default_value = RPC_API_URL)]
    rpc_api_url: Url,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Set debug logging before initializing telemetry
    std::env::set_var("RUST_LOG", "debug");
    
    let _guard = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();

    let Args {
        db_args,
        mut indexer_args,
        metrics_address,
        remote_store_url,
        database_url,
        contract_package_id,
        rpc_api_url,
    } = Args::parse();

    let cancel = CancellationToken::new();
    let registry = Registry::new_custom(Some("suins".into()), None)
        .context("Failed to create Prometheus registry.")?;
    let metrics = MetricsService::new(
        MetricsArgs { metrics_address },
        registry,
        cancel.child_token(),
    );

    // Configure pipeline settings for rate limiting
    let pipeline_config = sui_indexer_alt_framework::pipeline::concurrent::ConcurrentConfig {
        committer: sui_indexer_alt_framework::pipeline::CommitterConfig {
            write_concurrency: 1,        // Reduce to single write
            collect_interval_ms: 1000,   // Collect more frequently
            watermark_interval_ms: 5000, // Update watermark much less frequently
        },
        pruner: None,
    };

    info!("Starting indexer with remote store URL: {}", remote_store_url);
    info!("Starting indexer with RPC URL: {}", rpc_api_url);
    info!("Contract package ID: {}", contract_package_id);

    // Configure client with rate limiting
    let client_args = ClientArgs {
        remote_store_url: Some(remote_store_url.clone()),
        local_ingestion_path: None,
        rpc_api_url: Some(rpc_api_url.clone()),
        rpc_username: None,
        rpc_password: None,
    };

    // Configure indexer with rate limiting 
    // indexer_args.first_checkpoint = Some(207052790);
    indexer_args.first_checkpoint = Some(207078439);
    indexer_args.last_checkpoint = None;
    indexer_args.skip_watermark = true;  // Skip watermark to reduce requests

    // Create a custom config with ingestion settings
    let mut config = sui_indexer_alt_framework::ingestion::IngestionConfig::default();
    config.checkpoint_buffer_size = 1;     // Process one checkpoint at a time
    config.ingest_concurrency = 1;         // Single ingestion thread
    config.retry_interval_ms = 1000;       // Longer delay between retries

    info!("Starting indexer with config: {:?}", config);
    info!("Starting from checkpoint: {}", indexer_args.first_checkpoint.unwrap_or(0));

    let mut indexer = Indexer::new(
        database_url.clone(),
        db_args,
        indexer_args,
        client_args,
        config,
        None,
        metrics.registry(),
        cancel.clone(),
    )
    .await?;

    let handler = OfferHandler::new(contract_package_id);

    info!("Starting pipeline with handler");
    // Run the pipeline with rate limiting configuration
    indexer
        .concurrent_pipeline(handler, pipeline_config)
        .await?;

    info!("Starting indexer run");
    let h_indexer = indexer.run().await?;
    let h_metrics = metrics.run().await?;

    let _ = h_indexer.await;
    cancel.cancel();
    let _ = h_metrics.await;

    Ok(())
}
