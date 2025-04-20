// OTP module declaration
pub mod hotp;
pub mod totp;

// Re-export specific items if needed.
// Assuming Hotp and Totp structs are used elsewhere via crate::otp::Hotp/Totp
#[allow(unused_imports)] // Used indirectly via main.rs
pub use hotp::Hotp;
#[allow(unused_imports)] // Used indirectly via main.rs
pub use totp::Totp;
