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

# Try to reuse the same OTP
echo -e "\n${BLUE}Testing OTP Reuse Prevention${NC}"
REUSE_RESPONSE=$(curl -s -X POST "${BASE_URL}/otp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$OTP\"}")
echo "$REUSE_RESPONSE" | jq

# Check if OTP reuse is rejected
REUSE_VALID=$(echo "$REUSE_RESPONSE" | jq -r '.valid')
if [ "$REUSE_VALID" = "false" ]; then
  echo -e "\nOTP Reuse Prevention: ${GREEN}Success${NC}"
else
  echo -e "\nOTP Reuse Prevention: ${RED}Failed${NC}"
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

echo ""
echo "================================"
echo -e "${BLUE}Testing HOTP Endpoints${NC}"
echo "================================"

# Define a counter for HOTP
COUNTER=1

# Generate an HOTP
echo -e "\n${BLUE}Generating HOTP (Counter: $COUNTER)${NC}"
HOTP_GEN_RESPONSE=$(curl -s -X POST "${BASE_URL}/hotp/generate" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"counter\": $COUNTER}")
echo "$HOTP_GEN_RESPONSE" | jq

# Extract HOTP from response
HOTP=$(echo "$HOTP_GEN_RESPONSE" | jq -r '.otp')

echo -e "\nHOTP: ${GREEN}$HOTP${NC}"

# Verify the HOTP
echo -e "\n${BLUE}Verifying HOTP (Counter: $COUNTER)${NC}"
HOTP_VERIFY_RESPONSE=$(curl -s -X POST "${BASE_URL}/hotp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$HOTP\", \"counter\": $COUNTER}")
echo "$HOTP_VERIFY_RESPONSE" | jq

# Check if HOTP is valid
HOTP_VALID=$(echo "$HOTP_VERIFY_RESPONSE" | jq -r '.valid')
if [ "$HOTP_VALID" = "true" ]; then
  echo -e "\nHOTP Verification: ${GREEN}Success${NC}"
else
  echo -e "\nHOTP Verification: ${RED}Failed${NC}"
fi

# Try to reuse the same HOTP + Counter
echo -e "\n${BLUE}Testing HOTP Reuse Prevention (Counter: $COUNTER)${NC}"
HOTP_REUSE_RESPONSE=$(curl -s -X POST "${BASE_URL}/hotp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$HOTP\", \"counter\": $COUNTER}")
echo "$HOTP_REUSE_RESPONSE" | jq

# Check if HOTP reuse is rejected
HOTP_REUSE_VALID=$(echo "$HOTP_REUSE_RESPONSE" | jq -r '.valid')
if [ "$HOTP_REUSE_VALID" = "false" ]; then
  echo -e "\nHOTP Reuse Prevention: ${GREEN}Success${NC}"
else
  echo -e "\nHOTP Reuse Prevention: ${RED}Failed${NC}"
fi

# Test with invalid HOTP
echo -e "\n${BLUE}Testing with Invalid HOTP (Counter: $COUNTER)${NC}"
INVALID_HOTP="invalid"
INVALID_HOTP_VERIFY_RESPONSE=$(curl -s -X POST "${BASE_URL}/hotp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$INVALID_HOTP\", \"counter\": $COUNTER}")
echo "$INVALID_HOTP_VERIFY_RESPONSE" | jq

# Check if invalid HOTP is rejected
INVALID_HOTP_VALID=$(echo "$INVALID_HOTP_VERIFY_RESPONSE" | jq -r '.valid')
if [ "$INVALID_HOTP_VALID" = "false" ]; then
  echo -e "\nInvalid HOTP Rejection: ${GREEN}Success${NC}"
else
  echo -e "\nInvalid HOTP Rejection: ${RED}Failed${NC}"
fi

# Test with different counter (should be valid if OTP matches that counter, but reuse prevention applies per counter)
COUNTER2=2
echo -e "\n${BLUE}Generating HOTP (Counter: $COUNTER2)${NC}"
HOTP_GEN2_RESPONSE=$(curl -s -X POST "${BASE_URL}/hotp/generate" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"counter\": $COUNTER2}")
echo "$HOTP_GEN2_RESPONSE" | jq
HOTP2=$(echo "$HOTP_GEN2_RESPONSE" | jq -r '.otp')
echo -e "\nHOTP2: ${GREEN}$HOTP2${NC}"

echo -e "\n${BLUE}Verifying HOTP (Counter: $COUNTER2)${NC}"
HOTP_VERIFY2_RESPONSE=$(curl -s -X POST "${BASE_URL}/hotp/verify" \
  -H "Content-Type: application/json" \
  -d "{\"secret\": \"$SECRET\", \"otp\": \"$HOTP2\", \"counter\": $COUNTER2}")
echo "$HOTP_VERIFY2_RESPONSE" | jq
HOTP2_VALID=$(echo "$HOTP_VERIFY2_RESPONSE" | jq -r '.valid')
if [ "$HOTP2_VALID" = "true" ]; then
  echo -e "\nHOTP Verification (Counter 2): ${GREEN}Success${NC}"
else
  echo -e "\nHOTP Verification (Counter 2): ${RED}Failed${NC}"
fi


echo -e "\n${BLUE}All Tests Complete${NC}"
