use crate::config::Config;
// Removed unused DashMap import
use redis::{AsyncCommands, Client as RedisClient};
use std::sync::Arc;
use std::time::Duration; // Removed unused Instant import
use tokio::time;

/// Storage trait for OTP storage backends
#[async_trait::async_trait]
pub trait OtpStore: Send + Sync {
    /// Mark an OTP as used
    async fn mark_used(&self, otp: &str, expiry_seconds: u64) -> Result<(), String>;

    /// Check if an OTP has been used
    async fn is_used(&self, otp: &str) -> Result<bool, String>;
}

/// Factory for creating OTP storage backends
pub struct OtpStorage;

impl OtpStorage {
    /// Create a new OTP storage backend based on configuration
    #[allow(clippy::new_ret_no_self)] // This is a factory function, not a constructor for OtpStorage
    pub async fn new(config: &Config) -> Result<Arc<dyn OtpStore>, String> {
        log::info!("Using Redis storage for OTPs at {}", config.redis_url);
        let store = RedisStore::new(&config.redis_url).await?;
        Ok(Arc::new(store))
    }
}

/// Redis storage for used OTPs
pub struct RedisStore {
    client: RedisClient,
}

impl RedisStore {
    /// Create a new Redis OTP storage with retry logic
    pub async fn new(redis_url: &str) -> Result<Self, String> {
        // Create Redis client
        let client = RedisClient::open(redis_url)
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;

        // Retry connection with exponential backoff
        let mut retry_count = 0;
        let max_retries = 5;
        let mut backoff_ms = 1000; // Start with 1 second

        loop {
            log::info!(
                "Attempting to connect to Redis (attempt {}/{})",
                retry_count + 1,
                max_retries
            );

            match client.get_async_connection().await {
                Ok(mut conn) => {
                    // Test connection with PING
                    match redis::cmd("PING").query_async::<_, ()>(&mut conn).await {
                        Ok(_) => {
                            log::info!("Successfully connected to Redis");
                            return Ok(Self { client });
                        }
                        Err(e) => {
                            log::warn!("Failed to ping Redis: {}", e);
                            if retry_count >= max_retries {
                                return Err(format!(
                                    "Failed to ping Redis after {} attempts: {}",
                                    max_retries, e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to connect to Redis: {}", e);
                    if retry_count >= max_retries {
                        return Err(format!(
                            "Failed to connect to Redis after {} attempts: {}",
                            max_retries, e
                        ));
                    }
                }
            }

            // Increment retry count
            retry_count += 1;

            // Sleep with exponential backoff
            log::info!("Waiting {}ms before retrying...", backoff_ms);
            time::sleep(Duration::from_millis(backoff_ms)).await;

            // Double the backoff time for next retry (exponential backoff)
            backoff_ms = std::cmp::min(backoff_ms * 2, 30000); // Cap at 30 seconds
        }
    }
}

#[async_trait::async_trait]
impl OtpStore for RedisStore {
    async fn mark_used(&self, otp: &str, expiry_seconds: u64) -> Result<(), String> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

        // Set key with expiration using raw command
        let _: () = redis::cmd("SETEX")
            .arg(format!("otp:{}", otp))
            .arg(expiry_seconds.to_string())
            .arg("1")
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Failed to set OTP in Redis: {}", e))?;

        Ok(())
    }

    async fn is_used(&self, otp: &str) -> Result<bool, String> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

        let exists: bool = conn
            .exists(format!("otp:{}", otp))
            .await
            .map_err(|e| format!("Failed to check OTP in Redis: {}", e))?;

        Ok(exists)
    }
}
