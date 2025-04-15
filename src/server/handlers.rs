use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::otp::totp::Totp;
use crate::storage::OtpStorage;
use actix_web::{web, HttpResponse};
use data_encoding::BASE32;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateSecretResponse {
    secret: String,
    secret_base32: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateOtpRequest {
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateOtpResponse {
    otp: String,
    expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    secret: String,
    otp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    valid: bool,
}

/// Generate a new random secret
pub async fn generate_secret() -> AppResult<HttpResponse> {
    let mut rng = rand::thread_rng();
    let mut secret = vec![0u8; 20];
    rng.fill(&mut secret[..]);
    
    // Encode the secret in base32 for easy sharing
    let secret_base32 = BASE32.encode(&secret);
    
    let response = GenerateSecretResponse {
        secret: hex::encode(&secret),
        secret_base32,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Generate an OTP for the given secret
pub async fn generate_otp(
    config: web::Data<Arc<Config>>,
    req: web::Json<GenerateOtpRequest>,
) -> AppResult<HttpResponse> {
    // Decode the secret from hex
    let secret = hex::decode(&req.secret)
        .map_err(|e| AppError::ValidationError(format!("Invalid secret: {}", e)))?;
    
    // Create a TOTP instance
    let totp = Totp::new(
        secret,
        config.otp_length,
        config.otp_expiry_seconds,
    );
    
    // Generate the OTP
    let otp = totp.generate()?;
    
    let response = GenerateOtpResponse {
        otp,
        expires_in: config.otp_expiry_seconds,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Verify an OTP against the given secret
pub async fn verify_otp(
    config: web::Data<Arc<Config>>,
    storage: web::Data<Arc<OtpStorage>>,
    req: web::Json<VerifyOtpRequest>,
) -> AppResult<HttpResponse> {
    // Check if OTP has been used before
    if storage.is_used(&req.otp) {
        log::warn!("OTP reuse attempt detected: {}", req.otp);
        return Ok(HttpResponse::Ok().json(VerifyOtpResponse { valid: false }));
    }
    
    // Decode the secret from hex
    let secret = hex::decode(&req.secret)
        .map_err(|e| AppError::ValidationError(format!("Invalid secret: {}", e)))?;
    
    // Create a TOTP instance
    let totp = Totp::new(
        secret,
        config.otp_length,
        config.otp_expiry_seconds,
    );
    
    // Verify the OTP
    let valid = totp.verify(&req.otp)?;
    
    // If OTP is valid, mark it as used
    if valid {
        storage.mark_used(&req.otp, config.otp_expiry_seconds);
        log::debug!("OTP marked as used: {}", req.otp);
    }
    
    let response = VerifyOtpResponse { valid };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Health check endpoint
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
