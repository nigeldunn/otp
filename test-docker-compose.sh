#!/bin/bash

# Script to test the OTP server running in Docker Compose

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed or not in PATH"
    echo "Please install jq to parse JSON responses: https://stedolan.github.io/jq/download/"
    exit 1
fi

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

BASE_URL="http://localhost:8080/api"

echo -e "${BLUE}Testing OTP Server in Docker Compose${NC}"
echo "======================================"

# Check if Docker Compose is running
echo -e "\n${BLUE}Checking if OTP server is running...${NC}"
if ! curl -s "$BASE_URL/health" > /dev/null; then
  echo -e "${RED}Error: OTP server is not running or not accessible${NC}"
  echo "Start the server with: ./run-docker-compose.sh"
  exit 1
fi

echo -e "${GREEN}OTP server is running${NC}"

# Test health endpoint
echo -e "\n${BLUE}Testing Health Endpoint${NC}"
HEALTH_RESPONSE=$(curl -s "$BASE_URL/health")
echo "$HEALTH_RESPONSE" | jq || echo "$HEALTH_RESPONSE"

# Generate a secret
echo -e "\n${BLUE}Generating Secret${NC}"
SECRET_RESPONSE=$(curl -s -X POST "$BASE_URL/secret")
echo "$SECRET_RESPONSE" | jq || echo "$SECRET_RESPONSE"

# Extract secret from response
SECRET=$(echo "$SECRET_RESPONSE" | jq -r '.secret')
SECRET_BASE32=$(echo "$SECRET_RESPONSE" | jq -r '.secret_base32')

echo -e "\nSecret: ${GREEN}$SECRET${NC}"
echo -e "Secret (Base32): ${GREEN}$SECRET_BASE32${NC}"

# Generate an OTP
echo -e "\n${BLUE}Generating OTP${NC}"
OTP_RESPONSE=$(curl -s -X POST "$BASE_URL/otp/generate" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\"}")
echo "$OTP_RESPONSE" | jq || echo "$OTP_RESPONSE"

# Extract OTP from response
OTP=$(echo "$OTP_RESPONSE" | jq -r '.otp')
EXPIRES_IN=$(echo "$OTP_RESPONSE" | jq -r '.expires_in')

echo -e "\nOTP: ${GREEN}$OTP${NC}"
echo -e "Expires in: ${GREEN}$EXPIRES_IN seconds${NC}"

# Verify the OTP
echo -e "\n${BLUE}Verifying OTP${NC}"
VERIFY_RESPONSE=$(curl -s -X POST "$BASE_URL/otp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$OTP\"}")
echo "$VERIFY_RESPONSE" | jq || echo "$VERIFY_RESPONSE"

# Check if OTP is valid
VALID=$(echo "$VERIFY_RESPONSE" | jq -r '.valid')
if [ "$VALID" = "true" ]; then
  echo -e "\nOTP Verification: ${GREEN}Success${NC}"
else
  echo -e "\nOTP Verification: ${RED}Failed${NC}"
  exit 1
fi

# Try to reuse the same OTP
echo -e "\n${BLUE}Testing OTP Reuse Prevention${NC}"
REUSE_RESPONSE=$(curl -s -X POST "$BASE_URL/otp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$OTP\"}")
echo "$REUSE_RESPONSE" | jq || echo "$REUSE_RESPONSE"

# Check if OTP reuse is rejected
REUSE_VALID=$(echo "$REUSE_RESPONSE" | jq -r '.valid')
if [ "$REUSE_VALID" = "false" ]; then
  echo -e "\nOTP Reuse Prevention: ${GREEN}Success${NC}"
else
  echo -e "\nOTP Reuse Prevention: ${RED}Failed${NC}"
  exit 1
fi

# Test with invalid OTP
echo -e "\n${BLUE}Testing with Invalid OTP${NC}"
INVALID_OTP="invalid"
INVALID_VERIFY_RESPONSE=$(curl -s -X POST "$BASE_URL/otp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$INVALID_OTP\"}")
echo "$INVALID_VERIFY_RESPONSE" | jq || echo "$INVALID_VERIFY_RESPONSE"

# Check if invalid OTP is rejected
INVALID_VALID=$(echo "$INVALID_VERIFY_RESPONSE" | jq -r '.valid')
if [ "$INVALID_VALID" = "false" ]; then
  echo -e "\nInvalid OTP Rejection: ${GREEN}Success${NC}"
else
  echo -e "\nInvalid OTP Rejection: ${RED}Failed${NC}"
  exit 1
fi

echo -e "\n${GREEN}All tests passed successfully!${NC}"
echo -e "The OTP server is working correctly with Redis storage."
