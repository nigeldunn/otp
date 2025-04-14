use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,
    pub otp_length: usize,
    pub otp_expiry_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            otp_length: 6,
            otp_expiry_seconds: 30,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let otp_length = env::var("OTP_LENGTH")
            .unwrap_or_else(|_| "6".to_string())
            .parse()
            .unwrap_or(6);
        let otp_expiry_seconds = env::var("OTP_EXPIRY_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Self {
            server_host,
            server_port,
            log_level,
            otp_length,
            otp_expiry_seconds,
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
