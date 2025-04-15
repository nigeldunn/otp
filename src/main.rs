mod config;
mod error;
mod otp;
mod server;
mod storage;

use actix_web::{App, HttpServer};
use config::Config;
use dotenv::dotenv;
use env_logger::Env;
use std::sync::Arc;
use storage::OtpStorage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print a message to stderr for debugging
    eprintln!("Starting OTP server application...");
    
    // Load environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Print environment variables for debugging
    eprintln!("Environment variables:");
    for (key, value) in std::env::vars() {
        eprintln!("  {}={}", key, value);
    }
    
    // Load configuration
    let config = Arc::new(Config::from_env());
    let server_address = config.server_address();
    
    eprintln!("Server address: {}", server_address);
    eprintln!("Storage type: {:?}", config.storage_type);
    
    // Initialize OTP storage
    eprintln!("Initializing OTP storage...");
    let otp_storage = match OtpStorage::new(&config).await {
        Ok(storage) => {
            eprintln!("OTP storage initialized successfully");
            storage
        },
        Err(e) => {
            eprintln!("Failed to initialize OTP storage: {}", e);
            log::error!("Failed to initialize OTP storage: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };
    
    log::info!("Starting OTP server on {}", server_address);
    eprintln!("Starting HTTP server on {}", server_address);
    
    // Start HTTP server
    eprintln!("Creating HTTP server...");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(config.clone()))
            .app_data(actix_web::web::Data::new(otp_storage.clone()))
            .configure(server::routes::configure_routes)
    })
    .bind(server_address);
    
    match server {
        Ok(server) => {
            eprintln!("HTTP server created successfully, starting...");
            server.run().await
        },
        Err(e) => {
            eprintln!("Failed to bind HTTP server: {}", e);
            Err(e)
        }
    }
}
