use clap::Parser;
use log::info;
use sui_indexer_alt_framework::cluster;
use sui_indexer_alt_framework::cluster::IndexerCluster;
use sui_indexer_alt_framework::pipeline::concurrent::ConcurrentConfig;
use suins_indexer::handlers::offer_events_handler::OfferEventsHandlerPipeline;
use suins_indexer::handlers::offers_handler::OffersHandlerPipeline;
use suins_indexer::MIGRATIONS;
use url::Url;

#[derive(clap::Parser, Debug)]
struct AppArgs {
    #[clap(long, env = "DATABASE_URL")]
    database_url: Url,

    #[clap(long, env = "CONTRACT_PACKAGE_ID")]
    contract_package_id: String,

    #[clap(flatten)]
    cluster_args: cluster::Args,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;
    env_logger::init();

    let args = AppArgs::parse();

    info!(
        "Starting indexer with Contract package ID: {}",
        args.contract_package_id
    );

    let mut indexer =
        IndexerCluster::new(args.database_url, args.cluster_args, Some(&MIGRATIONS)).await?;

    info!("Starting pipeline with handler");

    // Process all offer events and save them to database to separate tables
    indexer
        .concurrent_pipeline(
            OfferEventsHandlerPipeline::new(args.contract_package_id.clone()),
            ConcurrentConfig::default(),
        )
        .await?;

    // Process all offer events and save up to date offer information in database
    indexer
        .concurrent_pipeline(
            OffersHandlerPipeline::new(args.contract_package_id),
            ConcurrentConfig::default(),
        )
        .await?;

    let _ = indexer.run().await?.await;

    Ok(())
}
