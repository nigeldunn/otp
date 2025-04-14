mod config;
mod error;
mod otp;
mod server;

use actix_web::{App, HttpServer};
use config::Config;
use dotenv::dotenv;
use env_logger::Env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Load configuration
    let config = Arc::new(Config::from_env());
    let server_address = config.server_address();
    
    log::info!("Starting OTP server on {}", server_address);
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(config.clone()))
            .configure(server::routes::configure_routes)
    })
    .bind(server_address)?
    .run()
    .await
}
