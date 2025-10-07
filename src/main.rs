use clap::Parser;
use log::info;
use std::fs;
use std::path::PathBuf;
use sui_indexer_alt_framework::cluster;
use sui_indexer_alt_framework::cluster::IndexerCluster;
use sui_indexer_alt_framework::pipeline::concurrent::ConcurrentConfig;
use sui_indexer_alt_framework::pipeline::sequential::SequentialConfig;
use sui_indexer_alt_framework::postgres::DbArgs;
use suins_indexer::handlers::auctions_handler::AuctionsHandlerPipeline;
use suins_indexer::handlers::offer_events_handler::OfferEventsHandlerPipeline;
use suins_indexer::handlers::offers_handler::OffersHandlerPipeline;
use suins_indexer::MIGRATIONS;
use url::Url;

#[derive(clap::Parser, Debug)]
struct AppArgs {
    #[clap(long, env = "DATABASE_URL")]
    database_url: Url,

    #[clap(long, env = "DATABASE_TLS_CA_CERT")]
    database_tls_ca_cert: Option<String>,

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

    let mut db_args = DbArgs::default();
    if let Some(cert_content) = args.database_tls_ca_cert {
        if !cert_content.is_empty() {
            let cert_dir = PathBuf::from("./certificates");
            fs::create_dir_all(&cert_dir)?;

            let cert_path = cert_dir.join("ca-cert.crt");
            fs::write(&cert_path, cert_content)?;

            db_args = DbArgs {
                tls_verify_cert: true,
                tls_ca_cert_path: Some(cert_path),
                ..DbArgs::default()
            };
        }
    }

    info!(
        "Starting indexer with Contract package ID: {}",
        args.contract_package_id
    );

    let mut indexer = IndexerCluster::builder()
        .with_database_url(args.database_url)
        .with_db_args(db_args)
        .with_args(args.cluster_args)
        .with_migrations(&MIGRATIONS)
        .build()
        .await?;

    info!("Starting pipeline with handler");

    // Process all offer events, in any order, and save them to database to separate tables
    indexer
        .concurrent_pipeline(
            OfferEventsHandlerPipeline::new(args.contract_package_id.clone()),
            ConcurrentConfig::default(),
        )
        .await?;

    // Process all offer events in order and save up to date offer information in database
    indexer
        .sequential_pipeline(
            OffersHandlerPipeline::new(args.contract_package_id.clone()),
            SequentialConfig::default(),
        )
        .await?;

    // Process all auction & bid events in order and save up to date offer information in database
    indexer
        .sequential_pipeline(
            AuctionsHandlerPipeline::new(args.contract_package_id),
            SequentialConfig::default(),
        )
        .await?;

    let _ = indexer.run().await?.await;

    Ok(())
}
