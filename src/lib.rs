use diesel_migrations::{embed_migrations, EmbeddedMigrations};

pub mod handlers;
pub mod models;
pub mod schema;

pub use handlers::offer_handler::OfferHandlerPipeline;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
