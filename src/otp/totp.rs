use crate::error::{AppError, AppResult};
use crate::otp::hotp::Hotp;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// TOTP (Time-based One-Time Password) implementation based on RFC6238
pub struct Totp {
    hotp: Hotp,
    time_step: u64,
    skew: u64,
}

impl Totp {
    /// Create a new TOTP instance with the given secret and configuration
    pub fn new(secret: Vec<u8>, digits: usize, time_step: u64) -> Self {
        let hotp = Hotp::new(secret, digits);
        Self {
            hotp,
            time_step,
            skew: 1, // Allow 1 step before and after for clock skew
        }
    }

    /// Set the allowed clock skew in time steps
    pub fn with_skew(mut self, skew: u64) -> Self {
        self.skew = skew;
        self
    }

    /// Get the current timestamp in seconds
    fn current_timestamp() -> AppResult<u64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|e| AppError::InternalError(format!("Time error: {}", e)))
    }

    /// Calculate the time counter based on the timestamp
    fn calculate_counter(&self, timestamp: u64) -> u64 {
        timestamp / self.time_step
    }

    /// Generate a TOTP code for the current time
    pub fn generate(&self) -> AppResult<String> {
        let timestamp = Self::current_timestamp()?;
        let counter = self.calculate_counter(timestamp);
        self.hotp.generate(counter)
    }

    /// Generate a TOTP code for a specific timestamp
    pub fn generate_at(&self, timestamp: u64) -> AppResult<String> {
        let counter = self.calculate_counter(timestamp);
        self.hotp.generate(counter)
    }

    /// Verify a TOTP code against the current time
    pub fn verify(&self, code: &str) -> AppResult<bool> {
        let timestamp = Self::current_timestamp()?;
        self.verify_at(code, timestamp)
    }

    /// Verify a TOTP code against a specific timestamp
    pub fn verify_at(&self, code: &str, timestamp: u64) -> AppResult<bool> {
        let counter = self.calculate_counter(timestamp);
        
        // Check current counter and allowed skew
        for i in counter.saturating_sub(self.skew)..=counter.saturating_add(self.skew) {
            if self.hotp.verify(code, i)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

impl fmt::Debug for Totp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Totp")
            .field("hotp", &self.hotp)
            .field("time_step", &self.time_step)
            .field("skew", &self.skew)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation() {
        let secret = b"12345678901234567890".to_vec();
        let totp = Totp::new(secret.clone(), 6, 30);
        
        // Test with specific timestamps
        let test_vectors = [
            (59, "287082"),          // 1970-01-01 00:00:59
            (1111111109, "081804"),  // 2005-03-18 01:58:29
            (1111111111, "050471"),  // 2005-03-18 01:58:31
            (1234567890, "005924"),  // 2009-02-13 23:31:30
            (2000000000, "279037"),  // 2033-05-18 03:33:20
        ];
        
        for (timestamp, expected) in test_vectors.iter() {
            let result = totp.generate_at(*timestamp).unwrap();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn test_totp_verification() {
        // Test basic verification
        let secret1 = b"12345678901234567890".to_vec();
        let totp = Totp::new(secret1, 6, 30);
        
        assert!(totp.verify_at("287082", 59).unwrap());
        assert!(totp.verify_at("081804", 1111111109).unwrap());
        assert!(!totp.verify_at("081804", 1111111169).unwrap()); // 60 seconds later, outside default skew
        
        // Test with skew
        let secret2 = b"12345678901234567890".to_vec();
        let totp_with_skew = Totp::new(secret2, 6, 30).with_skew(1);
        assert!(totp_with_skew.verify_at("081804", 1111111139).unwrap()); // 30 seconds later, within skew
        
        // Test without skew
        let secret3 = b"12345678901234567890".to_vec();
        let totp_no_skew = Totp::new(secret3, 6, 30).with_skew(0);
        assert!(!totp_no_skew.verify_at("081804", 1111111139).unwrap()); // 30 seconds later, outside skew
    }
}
