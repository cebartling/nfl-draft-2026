#!/bin/bash
set -e

# NFL Draft Frontend - Docker Build Script
# Builds the frontend Docker image with configurable options

# Default values
IMAGE_NAME="${DOCKER_IMAGE_NAME:-nfl-draft-frontend}"
IMAGE_TAG="${DOCKER_IMAGE_TAG:-latest}"
PLATFORM="${DOCKER_PLATFORM:-linux/amd64}"

echo "============================================"
echo "Building NFL Draft Frontend Docker Image"
echo "============================================"
echo ""
echo "Configuration:"
echo "  Image Name: ${IMAGE_NAME}"
echo "  Image Tag:  ${IMAGE_TAG}"
echo "  Platform:   ${PLATFORM}"
echo ""

# Build the Docker image
docker build \
    --platform "${PLATFORM}" \
    --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
    --tag "${IMAGE_NAME}:latest" \
    --progress=plain \
    .

echo ""
echo "============================================"
echo "✓ Docker image built successfully"
echo "============================================"
echo ""
echo "Image Details:"
docker images "${IMAGE_NAME}:${IMAGE_TAG}" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
echo ""
echo "Quick Start:"
echo "  # Run standalone (frontend only)"
echo "  docker run -p 3000:8080 ${IMAGE_NAME}:${IMAGE_TAG}"
echo ""
echo "  # Run full stack with docker compose"
echo "  cd .. && docker compose up frontend"
echo ""
echo "  # Run all services"
echo "  cd .. && docker compose up -d"
echo ""
