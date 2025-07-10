use clap::Parser;
use sui_indexer_alt_framework::cluster;
use sui_indexer_alt_framework::cluster::IndexerCluster;
use sui_indexer_alt_framework::pipeline::concurrent::ConcurrentConfig;
use suins_indexer::handlers::offer_handler::OfferHandlerPipeline;
use suins_indexer::MIGRATIONS;
use log::info;
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

    indexer
        .concurrent_pipeline(
            OfferHandlerPipeline::new(args.contract_package_id),
            ConcurrentConfig::default(),
        )
        .await?;

    let _ = indexer.run().await?.await;

    Ok(())
}
