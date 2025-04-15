#!/bin/bash

# Script to deploy the OTP server Helm chart to a Kubernetes cluster

# Default values
RELEASE_NAME="otp-server"
NAMESPACE="default"
VALUES_FILE=""
SET_VALUES=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --release)
      RELEASE_NAME="$2"
      shift 2
      ;;
    --namespace)
      NAMESPACE="$2"
      shift 2
      ;;
    --values)
      VALUES_FILE="$2"
      shift 2
      ;;
    --set)
      SET_VALUES="${SET_VALUES} --set $2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--release RELEASE_NAME] [--namespace NAMESPACE] [--values VALUES_FILE] [--set KEY=VALUE]"
      echo ""
      echo "Options:"
      echo "  --release RELEASE_NAME    Helm release name (default: otp-server)"
      echo "  --namespace NAMESPACE     Kubernetes namespace (default: default)"
      echo "  --values VALUES_FILE      Path to values file"
      echo "  --set KEY=VALUE           Set individual values (can be used multiple times)"
      echo "  --help                    Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Build the helm command
HELM_CMD="helm upgrade --install ${RELEASE_NAME} ./helm/otp-server --namespace ${NAMESPACE}"

# Add values file if provided
if [ -n "$VALUES_FILE" ]; then
  HELM_CMD="${HELM_CMD} -f ${VALUES_FILE}"
fi

# Add set values if provided
if [ -n "$SET_VALUES" ]; then
  HELM_CMD="${HELM_CMD} ${SET_VALUES}"
fi

# Execute the helm command
echo "Deploying OTP server to Kubernetes..."
echo "Command: ${HELM_CMD}"
eval "${HELM_CMD}"

echo "Done!"
