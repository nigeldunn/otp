# OTP Server

A horizontally scalable OTP (One-Time Password) server written in Rust, following the [RFC4226](https://datatracker.ietf.org/doc/html/rfc4226) (HOTP) and [RFC6238](https://datatracker.ietf.org/doc/html/rfc6238) (TOTP) standards.

## Features

- Generates 6-character long, numeric one-time passwords (configurable length)
- Supports both TOTP (Time-based) and HOTP (Counter-based) via separate API endpoints
- Prevents OTP reuse using Redis as the storage backend (tracks TOTP codes and HOTP code/counter pairs)
- RESTful API for easy integration
- Horizontally scalable architecture (requires Redis)
- Configurable via environment variables

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version) for local development
- Docker for containerized deployment
- Kubernetes and Helm for orchestrated deployment

### Local Installation

1. Clone the repository:
   ```
   git clone https://github.com/nigeldunn/otp-server.git
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

### Docker Deployment

1. Build the Docker image:
   ```
   docker build -t otp-server:latest .
   ```

2. Run the container:
   ```
   docker run -p 8080:8080 otp-server:latest
   ```

3. Or use the provided script to build and push to a registry:
   ```
   ./build-and-push.sh --registry your-registry --tag v1.0.0
   ```

### Docker Compose Deployment

Requires Docker Compose. This setup includes the OTP server and the required Redis instance:

1. Start the services:
   ```
   ./run-docker-compose.sh
   ```

2. Start in detached mode:
   ```
   ./run-docker-compose.sh --detach
   ```

3. Stop and remove the services:
   ```
   ./run-docker-compose.sh --down
   ```

4. Rebuild the services:
   ```
   ./run-docker-compose.sh --build
   ```

The script automatically detects whether you have the standalone `docker-compose` command or the newer `docker compose` plugin installed, making it compatible with all Docker installations.

The Docker Compose setup includes:
- OTP server (configured to use the Redis service)
- Redis instance 
- Proper networking between services

5. Test the Docker Compose deployment:
   ```
   ./test-docker-compose.sh
   ```
   This script will verify that the OTP server is running correctly with Redis storage by testing all API endpoints and verifying OTP reuse prevention.

### Kubernetes Deployment with Helm

1. Install the Helm chart:
   ```
   helm install otp-server ./helm/otp-server
   ```

2. Customize the deployment:
   ```
   helm install otp-server ./helm/otp-server --set service.type=LoadBalancer
   ```

3. Upgrade an existing deployment:
   ```
   helm upgrade otp-server ./helm/otp-server --set replicaCount=3
   ```

4. Or use the provided script for easier deployment:
   ```
   ./deploy-helm.sh --namespace otp --set image.repository=your-registry/otp-server --set image.tag=v1.0.0
   ```

### Configuration

The server can be configured using environment variables or a `.env` file:

- `SERVER_HOST`: Host address to bind to (default: 127.0.0.1)
- `SERVER_PORT`: Port to listen on (default: 8080)
- `LOG_LEVEL`: Logging level (default: info)
- `OTP_LENGTH`: Length of generated OTP codes (default: 6)
- `OTP_EXPIRY_SECONDS`: Validity period of OTP codes in seconds (default: 30). Used OTPs expire in Redis after this duration.
- `REDIS_URL`: Redis connection URL (default: redis://127.0.0.1:6379). This is required for the server to function.

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

### Generate TOTP (Time-Based)

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

### Verify TOTP (Time-Based)

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

### Generate HOTP (Counter-Based)

```
POST /api/hotp/generate
```

**Request:**
```json
{
  "secret": "hex_encoded_secret",
  "counter": 123 
}
```

**Response:**
```json
{
  "otp": "generated_hotp_code"
}
```

### Verify HOTP (Counter-Based)

```
POST /api/hotp/verify
```

**Request:**
```json
{
  "secret": "hex_encoded_secret",
  "otp": "hotp_code_to_verify",
  "counter": 123
}
```

**Response:**
```json
{
  "valid": true
}
```

## Development

### Continuous Integration and Deployment

This project uses GitHub Actions for CI/CD:

#### CI Workflow (`.github/workflows/ci.yml`)
- Running tests on every push and pull request
- Linting with Clippy and checking code formatting
- Building the project and uploading artifacts
- Building the Docker image

#### Release Workflow (`.github/workflows/release.yml`)
- Triggered when a new tag is pushed
- Creates a GitHub release
- Builds binaries for multiple platforms (Linux, macOS, both x86_64 and ARM64)
- Builds and pushes Docker images to GitHub Container Registry

### Dependency Management

This project uses GitHub's Dependabot to keep dependencies up to date. The configuration is in `.github/dependabot.yml` and includes:

- Weekly updates for Rust dependencies (Cargo)
- Weekly updates for Docker images
- Monthly updates for GitHub Actions
- Weekly updates for Helm charts

Dependabot will automatically create pull requests when new versions of dependencies are available.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
