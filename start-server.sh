#!/bin/bash
set -e

echo "Starting OTP Server..."

# Function to handle shutdown signals
shutdown() {
  echo "Received shutdown signal, shutting down gracefully..."
  kill -TERM "$child" 2>/dev/null
  wait "$child"
  echo "Server shutdown complete"
  exit 0
}

# Set up signal handlers
trap shutdown SIGTERM SIGINT

# Start the OTP server
echo "Executing OTP server..."
./otp &

# Store the PID
child=$!

# Wait for the server process to terminate
wait "$child"

# If we get here, the server exited on its own
exit_code=$?
echo "OTP server exited with code $exit_code"
exit $exit_code
