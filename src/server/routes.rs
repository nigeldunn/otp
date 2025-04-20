use crate::server::handlers;
use actix_web::web;

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health_check))
            .route("/secret", web::post().to(handlers::generate_secret))
            // TOTP routes (prefixed with /otp for clarity, could be /totp)
            .route("/otp/generate", web::post().to(handlers::generate_otp))
            .route("/otp/verify", web::post().to(handlers::verify_otp))
            // HOTP routes
            .route("/hotp/generate", web::post().to(handlers::generate_hotp))
            .route("/hotp/verify", web::post().to(handlers::verify_hotp)),
    );
}
