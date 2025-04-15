#!/bin/bash

# Script to build and push the OTP server Docker image

# Default values
IMAGE_NAME="otp-server"
TAG="latest"
REGISTRY=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --registry)
      REGISTRY="$2/"
      shift 2
      ;;
    --tag)
      TAG="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--registry REGISTRY] [--tag TAG]"
      echo ""
      echo "Options:"
      echo "  --registry REGISTRY  Docker registry to push to (e.g., docker.io/username)"
      echo "  --tag TAG           Image tag (default: latest)"
      echo "  --help              Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Full image name with registry and tag
FULL_IMAGE_NAME="${REGISTRY}${IMAGE_NAME}:${TAG}"

echo "Building Docker image: ${FULL_IMAGE_NAME}"
docker build -t "${FULL_IMAGE_NAME}" .

# If registry is provided, push the image
if [ -n "$REGISTRY" ]; then
  echo "Pushing Docker image to registry: ${FULL_IMAGE_NAME}"
  docker push "${FULL_IMAGE_NAME}"
else
  echo "No registry provided. Skipping push."
  echo "To push to a registry, use: $0 --registry your-registry"
fi

echo "Done!"
