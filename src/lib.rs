use diesel_migrations::{embed_migrations, EmbeddedMigrations};

pub mod handlers;
pub mod models;
pub mod schema;

pub use handlers::offer_handler::OfferHandler;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Environment constants
pub const CONTRACT_PACKAGE_ID: &str = "0xe42285c9bfdda621f8164264223c231ecd1818c6dff8af962ab9e21f5877078b";
pub const DATABASE_URL: &str = "postgres://root:password@localhost:5432/microservice";
pub const METRICS_ADDRESS: &str = "0.0.0.0:9184";
pub const RPC_API_URL: &str = "https://fullnode.testnet.sui.io:443";
pub const REMOTE_STORE_URL: &str = "https://checkpoints.testnet.sui.io";
