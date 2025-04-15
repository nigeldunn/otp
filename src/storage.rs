use crate::config::{Config, StorageType};
use dashmap::DashMap;
use redis::{AsyncCommands, Client as RedisClient};
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    pub async fn new(config: &Config) -> Result<Arc<dyn OtpStore>, String> {
        match config.storage_type {
            StorageType::InMemory => {
                log::info!("Using in-memory storage for OTPs");
                Ok(Arc::new(InMemoryStore::new(config.storage_cleanup_interval)))
            },
            StorageType::Redis => {
                log::info!("Using Redis storage for OTPs at {}", config.redis_url);
                match RedisStore::new(&config.redis_url).await {
                    Ok(store) => Ok(Arc::new(store)),
                    Err(e) => {
                        log::warn!("Failed to connect to Redis: {}. Falling back to in-memory storage", e);
                        Ok(Arc::new(InMemoryStore::new(config.storage_cleanup_interval)))
                    }
                }
            }
        }
    }
}

/// In-memory storage for used OTPs
pub struct InMemoryStore {
    /// Map of used OTPs with their expiration time
    used_otps: DashMap<String, Instant>,
    /// Cleanup interval in seconds
    cleanup_interval: u64,
}

impl InMemoryStore {
    /// Create a new in-memory OTP storage
    pub fn new(cleanup_interval: u64) -> Self {
        let store = Self {
            used_otps: DashMap::new(),
            cleanup_interval,
        };
        
        // Start background cleanup task
        Self::start_cleanup_task(store.used_otps.clone(), cleanup_interval);
        
        store
    }
    
    /// Start a background task to clean up expired OTPs
    fn start_cleanup_task(used_otps: DashMap<String, Instant>, cleanup_interval: u64) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(cleanup_interval));
            
            loop {
                interval.tick().await;
                let now = Instant::now();
                used_otps.retain(|_, expiry| *expiry > now);
                log::debug!("Cleaned up expired OTPs from in-memory storage");
            }
        });
    }
}

#[async_trait::async_trait]
impl OtpStore for InMemoryStore {
    async fn mark_used(&self, otp: &str, expiry_seconds: u64) -> Result<(), String> {
        let expiry = Instant::now() + Duration::from_secs(expiry_seconds);
        self.used_otps.insert(otp.to_string(), expiry);
        Ok(())
    }
    
    async fn is_used(&self, otp: &str) -> Result<bool, String> {
        Ok(self.used_otps.contains_key(otp))
    }
}

/// Redis storage for used OTPs
pub struct RedisStore {
    client: RedisClient,
}

impl RedisStore {
    /// Create a new Redis OTP storage
    pub async fn new(redis_url: &str) -> Result<Self, String> {
        let client = RedisClient::open(redis_url)
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;
        
        // Test connection
        let mut conn = client.get_async_connection()
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;
        
        let _: () = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Failed to ping Redis: {}", e))?;
        
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl OtpStore for RedisStore {
    async fn mark_used(&self, otp: &str, expiry_seconds: u64) -> Result<(), String> {
        let mut conn = self.client.get_async_connection()
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
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;
        
        let exists: bool = conn.exists(format!("otp:{}", otp))
            .await
            .map_err(|e| format!("Failed to check OTP in Redis: {}", e))?;
        
        Ok(exists)
    }
}
