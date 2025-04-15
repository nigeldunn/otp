#!/bin/bash

# Script to run the OTP server with Docker Compose

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}OTP Server Docker Compose Runner${NC}"
echo "======================================="

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed or not in PATH${NC}"
    exit 1
fi

# Check if Docker Compose is installed (either as docker-compose or docker compose)
DOCKER_COMPOSE_CMD=""
if command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE_CMD="docker-compose"
elif command -v docker &> /dev/null && docker compose version &> /dev/null; then
    DOCKER_COMPOSE_CMD="docker compose"
else
    echo -e "${RED}Error: Docker Compose is not installed or not in PATH${NC}"
    echo "Please install Docker Compose or make sure Docker CLI with compose plugin is available."
    exit 1
fi

echo -e "${BLUE}Using Docker Compose command: ${GREEN}${DOCKER_COMPOSE_CMD}${NC}"

# Parse command line arguments
ACTION="up"
DETACHED=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --down)
      ACTION="down"
      shift
      ;;
    --restart)
      ACTION="restart"
      shift
      ;;
    --build)
      ACTION="build"
      shift
      ;;
    --detach|-d)
      DETACHED=true
      shift
      ;;
    --help|-h)
      echo "Usage: $0 [--down] [--restart] [--build] [--detach|-d]"
      echo ""
      echo "Options:"
      echo "  --down      Stop and remove containers, networks, and volumes"
      echo "  --restart   Restart all services"
      echo "  --build     Build or rebuild services"
      echo "  --detach    Run containers in the background"
      echo "  --help      Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help to see available options"
      exit 1
      ;;
  esac
done

# Execute the appropriate Docker Compose command
case $ACTION in
  "up")
    echo -e "${BLUE}Starting OTP server and Redis...${NC}"
    if [ "$DETACHED" = true ]; then
      $DOCKER_COMPOSE_CMD up -d
      echo -e "${GREEN}Services started in the background${NC}"
      echo -e "Access the OTP server at ${GREEN}http://localhost:8080${NC}"
    else
      echo -e "Press Ctrl+C to stop the services"
      $DOCKER_COMPOSE_CMD up
    fi
    ;;
  "down")
    echo -e "${BLUE}Stopping and removing containers...${NC}"
    $DOCKER_COMPOSE_CMD down
    echo -e "${GREEN}Services stopped and removed${NC}"
    ;;
  "restart")
    echo -e "${BLUE}Restarting services...${NC}"
    $DOCKER_COMPOSE_CMD restart
    echo -e "${GREEN}Services restarted${NC}"
    ;;
  "build")
    echo -e "${BLUE}Building services...${NC}"
    $DOCKER_COMPOSE_CMD build
    echo -e "${GREEN}Build complete${NC}"
    ;;
esac
