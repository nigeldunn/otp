use crate::error::{AppError, AppResult};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::fmt;

type HmacSha1 = Hmac<Sha1>;

/// HOTP (HMAC-based One-Time Password) implementation based on RFC4226
pub struct Hotp {
    secret: Vec<u8>,
    digits: usize,
    alphabet: String,
}

impl Hotp {
    /// Create a new HOTP instance with the given secret and configuration
    pub fn new(secret: Vec<u8>, digits: usize) -> Self {
        // Default to numeric characters (0-9) for compatibility with standard OTP implementations
        let alphabet = "0123456789".to_string();

        Self {
            secret,
            digits,
            alphabet,
        }
    }

    // /// Create a new HOTP instance with a custom alphabet
    // pub fn with_alphabet(secret: Vec<u8>, digits: usize, alphabet: String) -> Self {
    //     Self {
    //         secret,
    //         digits,
    //         alphabet,
    //     }
    // }

    /// Generate an HOTP code for the given counter value
    pub fn generate(&self, counter: u64) -> AppResult<String> {
        // Convert counter to big-endian byte array
        let counter_bytes = counter.to_be_bytes();

        // Create HMAC instance
        let mut mac = HmacSha1::new_from_slice(&self.secret)
            .map_err(|e| AppError::Internal(format!("HMAC error: {}", e)))?; // Updated to AppError::Internal

        // Update HMAC with counter bytes
        mac.update(&counter_bytes);

        // Finalize and get the result
        let result = mac.finalize().into_bytes();

        // Dynamic truncation as per RFC 4226
        let offset = (result[result.len() - 1] & 0xf) as usize;

        // Get 4 bytes from the result starting at the offset
        let binary = ((result[offset] & 0x7f) as u32) << 24
            | (result[offset + 1] as u32) << 16
            | (result[offset + 2] as u32) << 8
            | (result[offset + 3] as u32);

        // Convert to the desired number of digits using the alphabet
        let base = self.alphabet.len() as u32;
        let mut code = String::new();
        let mut value = binary % base.pow(self.digits as u32);

        for _ in 0..self.digits {
            let idx = (value % base) as usize;
            code.insert(0, self.alphabet.chars().nth(idx).unwrap());
            value /= base;
        }

        Ok(code)
    }

    /// Verify an HOTP code against the given counter value
    pub fn verify(&self, code: &str, counter: u64) -> AppResult<bool> {
        let generated = self.generate(counter)?;
        Ok(generated == code.to_lowercase())
    }
}

impl fmt::Debug for Hotp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hotp")
            .field("digits", &self.digits)
            .field("alphabet_length", &self.alphabet.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotp_generation() {
        let secret = b"12345678901234567890".to_vec();
        let hotp = Hotp::new(secret, 6);

        // Test vectors from RFC 4226
        let test_vectors = [
            (0, "755224"),
            (1, "287082"),
            (2, "359152"),
            (3, "969429"),
            (4, "338314"),
            (5, "254676"),
            (6, "287922"),
            (7, "162583"),
            (8, "399871"),
            (9, "520489"),
        ];

        for (counter, expected) in test_vectors.iter() {
            let result = hotp.generate(*counter).unwrap();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn test_hotp_verification() {
        let secret = b"12345678901234567890".to_vec();
        let hotp = Hotp::new(secret, 6);

        assert!(hotp.verify("755224", 0).unwrap());
        assert!(hotp.verify("287082", 1).unwrap());
        assert!(!hotp.verify("287082", 2).unwrap());
    }
}
