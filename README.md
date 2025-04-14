# OTP Server

[!CAUTION]
This server was built entirely by AI. I have not yet verified how it has implemented the RFCs yet. Use at your own risk!

A horizontally scalable OTP (One-Time Password) server written in Rust, following the [RFC4226](https://datatracker.ietf.org/doc/html/rfc4226) (HOTP) and [RFC6238](https://datatracker.ietf.org/doc/html/rfc6238) (TOTP) standards.

## Features

- Generates 6-character long, alphanumeric one-time passwords
- Supports both HOTP (HMAC-based One-Time Password) and TOTP (Time-based One-Time Password)
- RESTful API for easy integration
- Horizontally scalable architecture
- Configurable via environment variables

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/otp-server.git
   cd otp-server
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the server:
   ```
   ./target/release/otp
   ```

### Configuration

The server can be configured using environment variables or a `.env` file:

- `SERVER_HOST`: Host address to bind to (default: 127.0.0.1)
- `SERVER_PORT`: Port to listen on (default: 8080)
- `LOG_LEVEL`: Logging level (default: info)
- `OTP_LENGTH`: Length of generated OTP codes (default: 6)
- `OTP_EXPIRY_SECONDS`: Validity period of OTP codes in seconds (default: 30)

## API Endpoints

### Health Check

```
GET /api/health
```

Returns the server status and version.

### Generate Secret

```
POST /api/secret
```

Generates a new random secret for OTP generation.

**Response:**
```json
{
  "secret": "hex_encoded_secret",
  "secret_base32": "base32_encoded_secret"
}
```

### Generate OTP

```
POST /api/otp/generate
```

**Request:**
```json
{
  "secret": "hex_encoded_secret"
}
```

**Response:**
```json
{
  "otp": "generated_otp_code",
  "expires_in": 30
}
```

### Verify OTP

```
POST /api/otp/verify
```

**Request:**
```json
{
  "secret": "hex_encoded_secret",
  "otp": "otp_code_to_verify"
}
```

**Response:**
```json
{
  "valid": true
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
