use anyhow::Result;
use tracing_subscriber;

mod analyzers;
mod api;
mod config;
mod models;
mod utils;

use api::create_app;
use config::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::load()?;
    
    // Create the application
    let app = create_app(config.clone()).await?;
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(&config.server.bind_address).await?;
    tracing::info!("Content Moderation System starting on {}", config.server.bind_address);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}