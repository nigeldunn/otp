use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

/// Storage for used OTPs to prevent replay attacks
pub struct OtpStorage {
    /// Map of used OTPs with their expiration time
    used_otps: DashMap<String, Instant>,
    /// Cleanup interval in seconds
    cleanup_interval: u64,
}

impl OtpStorage {
    /// Create a new OTP storage
    pub fn new(cleanup_interval: u64) -> Arc<Self> {
        let storage = Arc::new(Self {
            used_otps: DashMap::new(),
            cleanup_interval,
        });
        
        // Start background cleanup task
        Self::start_cleanup_task(Arc::clone(&storage));
        
        storage
    }
    
    /// Start a background task to clean up expired OTPs
    fn start_cleanup_task(storage: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(storage.cleanup_interval));
            
            loop {
                interval.tick().await;
                storage.cleanup_expired();
                log::debug!("Cleaned up expired OTPs");
            }
        });
    }
    
    /// Clean up expired OTPs
    fn cleanup_expired(&self) {
        let now = Instant::now();
        self.used_otps.retain(|_, expiry| *expiry > now);
    }
    
    /// Mark an OTP as used
    pub fn mark_used(&self, otp: &str, expiry_seconds: u64) {
        let expiry = Instant::now() + Duration::from_secs(expiry_seconds);
        self.used_otps.insert(otp.to_string(), expiry);
    }
    
    /// Check if an OTP has been used
    pub fn is_used(&self, otp: &str) -> bool {
        self.used_otps.contains_key(otp)
    }
}
