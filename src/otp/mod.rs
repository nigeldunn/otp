// OTP module declaration
pub mod hotp;
pub mod totp;

// Re-export commonly used items
pub use hotp::*;
pub use totp::*;
