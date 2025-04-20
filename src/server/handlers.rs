use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::otp::{hotp::Hotp, totp::Totp}; // Import Hotp
use crate::storage::OtpStore;
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

// --- HOTP Structs ---
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateHotpRequest {
    secret: String,
    counter: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateHotpResponse {
    otp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct VerifyHotpRequest {
    secret: String,
    otp: String,
    counter: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyHotpResponse {
    valid: bool,
}
// --- End HOTP Structs ---

/// Generate a new random secret
pub async fn generate_secret() -> AppResult<HttpResponse> {
    // Use recommended way to get thread-local RNG
    let mut rng = rand::rngs::ThreadRng::default();
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
        .map_err(|e| AppError::Validation(format!("Invalid secret: {}", e)))?; // Updated to AppError::Validation

    // Create a TOTP instance
    let totp = Totp::new(secret, config.otp_length, config.otp_expiry_seconds);

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
    storage: web::Data<Arc<dyn OtpStore>>,
    req: web::Json<VerifyOtpRequest>,
) -> AppResult<HttpResponse> {
    // Check if OTP has been used before
    let is_used = storage
        .is_used(&req.otp)
        .await
        .map_err(|e| AppError::Internal(format!("Storage error: {}", e)))?; // Updated to AppError::Internal

    if is_used {
        log::warn!("OTP reuse attempt detected: {}", req.otp);
        return Ok(HttpResponse::Ok().json(VerifyOtpResponse { valid: false }));
    }

    // Decode the secret from hex
    let secret = hex::decode(&req.secret)
        .map_err(|e| AppError::Validation(format!("Invalid secret: {}", e)))?; // Updated to AppError::Validation

    // Create a TOTP instance
    let totp = Totp::new(secret, config.otp_length, config.otp_expiry_seconds);

    // Verify the OTP
    let valid = totp.verify(&req.otp)?;

    // If OTP is valid, mark it as used
    if valid {
        storage
            .mark_used(&req.otp, config.otp_expiry_seconds)
            .await
            .map_err(|e| AppError::Internal(format!("Storage error: {}", e)))?; // Updated to AppError::Internal
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

// --- HOTP Handlers ---

/// Generate an HOTP for the given secret and counter
pub async fn generate_hotp(
    config: web::Data<Arc<Config>>,
    req: web::Json<GenerateHotpRequest>,
) -> AppResult<HttpResponse> {
    // Decode the secret from hex
    let secret = hex::decode(&req.secret)
        .map_err(|e| AppError::Validation(format!("Invalid secret: {}", e)))?;

    // Create an HOTP instance
    let hotp = Hotp::new(secret, config.otp_length);

    // Generate the HOTP
    let otp = hotp.generate(req.counter)?;

    let response = GenerateHotpResponse { otp };

    Ok(HttpResponse::Ok().json(response))
}

/// Verify an HOTP against the given secret and counter
pub async fn verify_hotp(
    config: web::Data<Arc<Config>>,
    storage: web::Data<Arc<dyn OtpStore>>,
    req: web::Json<VerifyHotpRequest>,
) -> AppResult<HttpResponse> {
    // Construct a unique key for HOTP reuse check (otp + counter) using hyphens
    let reuse_key = format!("hotp-{}-{}", req.otp, req.counter);

    // Check if this specific OTP+Counter combination has been used before
    let is_used = storage
        .is_used(&reuse_key)
        .await
        .map_err(|e| AppError::Internal(format!("Storage error: {}", e)))?;

    if is_used {
        log::warn!(
            "HOTP reuse attempt detected: otp={}, counter={}",
            req.otp,
            req.counter
        );
        return Ok(HttpResponse::Ok().json(VerifyHotpResponse { valid: false }));
    }

    // Decode the secret from hex
    let secret = hex::decode(&req.secret)
        .map_err(|e| AppError::Validation(format!("Invalid secret: {}", e)))?;

    // Create an HOTP instance
    let hotp = Hotp::new(secret, config.otp_length);

    // Verify the HOTP
    let valid = hotp.verify(&req.otp, req.counter)?;

    // If HOTP is valid, mark this OTP+Counter combination as used
    if valid {
        // Use OTP expiry seconds for consistency, although HOTP doesn't strictly expire
        storage
            .mark_used(&reuse_key, config.otp_expiry_seconds)
            .await
            .map_err(|e| AppError::Internal(format!("Storage error: {}", e)))?;
        log::debug!(
            "HOTP marked as used: otp={}, counter={}",
            req.otp,
            req.counter
        );
    }

    let response = VerifyHotpResponse { valid };

    Ok(HttpResponse::Ok().json(response))
}

// --- End HOTP Handlers ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use actix_web::{body::to_bytes, http::StatusCode, test, web, App}; // Added to_bytes
    use async_trait::async_trait;
    use dashmap::DashMap;
    use std::sync::Arc;

    // Mock OtpStore for testing handlers in isolation
    #[derive(Debug)]
    struct MockOtpStore {
        used_otps: DashMap<String, ()>,
    }

    impl MockOtpStore {
        fn new() -> Self {
            Self {
                used_otps: DashMap::new(),
            }
        }
    }

    #[async_trait]
    impl OtpStore for MockOtpStore {
        async fn mark_used(&self, otp_key: &str, _expiry_seconds: u64) -> Result<(), String> {
            self.used_otps.insert(otp_key.to_string(), ());
            Ok(())
        }

        async fn is_used(&self, otp_key: &str) -> Result<bool, String> {
            Ok(self.used_otps.contains_key(otp_key))
        }
    }

    // Helper to create default config for tests
    fn test_config() -> Config {
        Config {
            otp_length: 6,
            otp_expiry_seconds: 30,
            ..Config::default()
        }
    }

    #[actix_web::test]
    async fn test_generate_hotp_handler() {
        let config = web::Data::new(Arc::new(test_config()));
        // Secret "12345678901234567890" hex encoded
        let secret_hex = "3132333435363738393031323334353637383930";
        let counter = 1u64;
        let req_payload = GenerateHotpRequest {
            secret: secret_hex.to_string(),
            counter,
        };
        let req = web::Json(req_payload);

        let resp = generate_hotp(config, req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body: GenerateHotpResponse = serde_json::from_slice(&body_bytes).unwrap();
        // Expected OTP for secret "12345678901234567890" and counter 1 is "287082" (from RFC)
        assert_eq!(body.otp, "287082");
    }

    #[actix_web::test]
    async fn test_verify_hotp_handler_valid() {
        let config = web::Data::new(Arc::new(test_config()));
        let storage = web::Data::new(Arc::new(MockOtpStore::new()) as Arc<dyn OtpStore>);
        // Secret "12345678901234567890" hex encoded
        let secret_hex = "3132333435363738393031323334353637383930";
        let counter = 1u64;
        let otp = "287082"; // Valid OTP for counter 1

        let req_payload = VerifyHotpRequest {
            secret: secret_hex.to_string(),
            otp: otp.to_string(),
            counter,
        };
        let req = web::Json(req_payload);

        let resp = verify_hotp(config, storage.clone(), req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body: VerifyOtpResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert!(body.valid);

        // Verify it was marked as used
        let reuse_key = format!("hotp:{}:{}", otp, counter);
        assert!(storage.is_used(&reuse_key).await.unwrap());
    }

    #[actix_web::test]
    async fn test_verify_hotp_handler_invalid() {
        let config = web::Data::new(Arc::new(test_config()));
        let storage = web::Data::new(Arc::new(MockOtpStore::new()) as Arc<dyn OtpStore>);
        let secret_hex = "3132333435363738393031323334353637383930";
        let counter = 1u64;
        let otp = "111111"; // Invalid OTP

        let req_payload = VerifyHotpRequest {
            secret: secret_hex.to_string(),
            otp: otp.to_string(),
            counter,
        };
        let req = web::Json(req_payload);

        let resp = verify_hotp(config, storage.clone(), req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body: VerifyOtpResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert!(!body.valid);

        // Verify it was NOT marked as used
        let reuse_key = format!("hotp:{}:{}", otp, counter);
        assert!(!storage.is_used(&reuse_key).await.unwrap());
    }

    #[actix_web::test]
    async fn test_verify_hotp_handler_reuse() {
        let config = web::Data::new(Arc::new(test_config()));
        let storage = web::Data::new(Arc::new(MockOtpStore::new()) as Arc<dyn OtpStore>);
        let secret_hex = "3132333435363738393031323334353637383930";
        let counter = 1u64;
        let otp = "287082"; // Valid OTP for counter 1

        let req_payload = VerifyHotpRequest {
            secret: secret_hex.to_string(),
            otp: otp.to_string(),
            counter,
        };

        // First verification (should be valid)
        let resp1 = verify_hotp(config.clone(), storage.clone(), web::Json(req_payload.clone()))
            .await
            .unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);
        let body_bytes1 = to_bytes(resp1.into_body()).await.unwrap();
        let body1: VerifyOtpResponse = serde_json::from_slice(&body_bytes1).unwrap();
        assert!(body1.valid);

        // Second verification (should be invalid due to reuse)
        let resp2 = verify_hotp(config, storage, web::Json(req_payload)).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body_bytes2 = to_bytes(resp2.into_body()).await.unwrap();
        let body2: VerifyOtpResponse = serde_json::from_slice(&body_bytes2).unwrap();
        assert!(!body2.valid);
    }

    // --- Integration Tests ---

    #[actix_web::test]
    async fn test_hotp_endpoints_integration() {
        let config = Arc::new(test_config());
        let storage = Arc::new(MockOtpStore::new()) as Arc<dyn OtpStore>;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config.clone()))
                .app_data(web::Data::new(storage.clone()))
                .configure(crate::server::routes::configure_routes), // Use the actual routes config
        )
        .await;

        // 1. Generate Secret (to get a valid secret for testing)
        let req_secret = test::TestRequest::post().uri("/api/secret").to_request();
        let resp_secret: GenerateSecretResponse = test::call_and_read_body_json(&app, req_secret).await;
        let secret_hex = resp_secret.secret; // Use the generated secret

        // 2. Generate HOTP
        let counter = 5u64;
        let gen_payload = GenerateHotpRequest {
            secret: secret_hex.clone(),
            counter,
        };
        let req_gen = test::TestRequest::post()
            .uri("/api/hotp/generate")
            .set_json(&gen_payload)
            .to_request();
        let resp_gen: GenerateHotpResponse = test::call_and_read_body_json(&app, req_gen).await;
        let generated_otp = resp_gen.otp;
        assert_eq!(generated_otp.len(), config.otp_length); // Check length

        // 3. Verify HOTP (Valid)
        let verify_payload_valid = VerifyHotpRequest {
            secret: secret_hex.clone(),
            otp: generated_otp.clone(),
            counter,
        };
        let req_verify_valid = test::TestRequest::post()
            .uri("/api/hotp/verify")
            .set_json(&verify_payload_valid)
            .to_request();
        let resp_verify_valid: VerifyOtpResponse =
            test::call_and_read_body_json(&app, req_verify_valid).await;
        assert!(resp_verify_valid.valid);

        // 4. Verify HOTP (Reuse - Invalid)
        let req_verify_reuse = test::TestRequest::post()
            .uri("/api/hotp/verify")
            .set_json(&verify_payload_valid) // Same payload as before
            .to_request();
        let resp_verify_reuse: VerifyOtpResponse =
            test::call_and_read_body_json(&app, req_verify_reuse).await;
        assert!(!resp_verify_reuse.valid);

        // 5. Verify HOTP (Incorrect OTP - Invalid)
        let verify_payload_invalid_otp = VerifyHotpRequest {
            secret: secret_hex.clone(),
            otp: "000000".to_string(), // Incorrect OTP
            counter,
        };
        let req_verify_invalid_otp = test::TestRequest::post()
            .uri("/api/hotp/verify")
            .set_json(&verify_payload_invalid_otp)
            .to_request();
        let resp_verify_invalid_otp: VerifyOtpResponse =
            test::call_and_read_body_json(&app, req_verify_invalid_otp).await;
        assert!(!resp_verify_invalid_otp.valid);

        // 6. Verify HOTP (Incorrect Counter - Invalid)
        let verify_payload_invalid_counter = VerifyHotpRequest {
            secret: secret_hex,
            otp: generated_otp,
            counter: counter + 1, // Incorrect counter
        };
        let req_verify_invalid_counter = test::TestRequest::post()
            .uri("/api/hotp/verify")
            .set_json(&verify_payload_invalid_counter)
            .to_request();
        let resp_verify_invalid_counter: VerifyOtpResponse =
            test::call_and_read_body_json(&app, req_verify_invalid_counter).await;
        assert!(!resp_verify_invalid_counter.valid);
    }
}
