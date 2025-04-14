#!/bin/bash

# Test script for OTP Server API

BASE_URL="http://127.0.0.1:8080/api"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Testing OTP Server API${NC}"
echo "================================"

# Test health endpoint
echo -e "\n${BLUE}Testing Health Endpoint${NC}"
curl -s "${BASE_URL}/health" | jq

# Generate a secret
echo -e "\n${BLUE}Generating Secret${NC}"
SECRET_RESPONSE=$(curl -s -X POST "${BASE_URL}/secret")
echo "$SECRET_RESPONSE" | jq

# Extract secret from response
SECRET=$(echo "$SECRET_RESPONSE" | jq -r '.secret')
SECRET_BASE32=$(echo "$SECRET_RESPONSE" | jq -r '.secret_base32')

echo -e "\nSecret: ${GREEN}$SECRET${NC}"
echo -e "Secret (Base32): ${GREEN}$SECRET_BASE32${NC}"

# Generate an OTP
echo -e "\n${BLUE}Generating OTP${NC}"
OTP_RESPONSE=$(curl -s -X POST "${BASE_URL}/otp/generate" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\"}")
echo "$OTP_RESPONSE" | jq

# Extract OTP from response
OTP=$(echo "$OTP_RESPONSE" | jq -r '.otp')
EXPIRES_IN=$(echo "$OTP_RESPONSE" | jq -r '.expires_in')

echo -e "\nOTP: ${GREEN}$OTP${NC}"
echo -e "Expires in: ${GREEN}$EXPIRES_IN seconds${NC}"

# Verify the OTP
echo -e "\n${BLUE}Verifying OTP${NC}"
VERIFY_RESPONSE=$(curl -s -X POST "${BASE_URL}/otp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$OTP\"}")
echo "$VERIFY_RESPONSE" | jq

# Check if OTP is valid
VALID=$(echo "$VERIFY_RESPONSE" | jq -r '.valid')
if [ "$VALID" = "true" ]; then
  echo -e "\nOTP Verification: ${GREEN}Success${NC}"
else
  echo -e "\nOTP Verification: ${RED}Failed${NC}"
fi

# Test with invalid OTP
echo -e "\n${BLUE}Testing with Invalid OTP${NC}"
INVALID_OTP="invalid"
INVALID_VERIFY_RESPONSE=$(curl -s -X POST "${BASE_URL}/otp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$INVALID_OTP\"}")
echo "$INVALID_VERIFY_RESPONSE" | jq

# Check if invalid OTP is rejected
INVALID_VALID=$(echo "$INVALID_VERIFY_RESPONSE" | jq -r '.valid')
if [ "$INVALID_VALID" = "false" ]; then
  echo -e "\nInvalid OTP Rejection: ${GREEN}Success${NC}"
else
  echo -e "\nInvalid OTP Rejection: ${RED}Failed${NC}"
fi

echo -e "\n${BLUE}Test Complete${NC}"
