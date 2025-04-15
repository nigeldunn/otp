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
    // Load environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Load configuration
    let config = Arc::new(Config::from_env());
    let server_address = config.server_address();
    
    // Initialize OTP storage
    let otp_storage = match OtpStorage::new(&config).await {
        Ok(storage) => storage,
        Err(e) => {
            log::error!("Failed to initialize OTP storage: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };
    
    log::info!("Starting OTP server on {}", server_address);
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(config.clone()))
            .app_data(actix_web::web::Data::new(otp_storage.clone()))
            .configure(server::routes::configure_routes)
    })
    .bind(server_address)?
    .run()
    .await
}
