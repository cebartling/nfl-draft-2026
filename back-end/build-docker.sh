#!/bin/bash
# Build script for NFL Draft Simulator Backend Docker image

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
IMAGE_NAME="${DOCKER_IMAGE_NAME:-nfl-draft-api}"
IMAGE_TAG="${DOCKER_IMAGE_TAG:-latest}"
PLATFORM="${DOCKER_PLATFORM:-linux/amd64}"

echo -e "${GREEN}Building NFL Draft Simulator Backend Docker Image${NC}"
echo "Image: ${IMAGE_NAME}:${IMAGE_TAG}"
echo "Platform: ${PLATFORM}"
echo ""

# Check if .sqlx directory exists
if [ ! -d ".sqlx" ]; then
    echo -e "${YELLOW}Warning: .sqlx directory not found${NC}"
    echo "SQLx offline mode requires .sqlx metadata for compile-time query verification"
    echo "Generate it by running:"
    echo "  cargo sqlx prepare --workspace"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build the Docker image
echo -e "${GREEN}Building Docker image...${NC}"
docker build \
    --platform "${PLATFORM}" \
    --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
    --tag "${IMAGE_NAME}:latest" \
    .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Docker image built successfully${NC}"
    echo ""
    echo "Image: ${IMAGE_NAME}:${IMAGE_TAG}"
    echo ""
    echo "To run the container:"
    echo "  docker run -p 8000:8000 --env-file .env ${IMAGE_NAME}:${IMAGE_TAG}"
    echo ""
    echo "Or use docker compose:"
    echo "  docker compose up api"
else
    echo -e "${RED}✗ Docker build failed${NC}"
    exit 1
fi
