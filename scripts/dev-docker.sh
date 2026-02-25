#!/usr/bin/env bash

# Docker-based Nix Development Environment for CW4626 Vault
set -e

echo "🐳 Starting Docker-based Nix environment for CW4626 Vault..."

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
  echo "❌ Docker is not running. Please start Docker Desktop first."
  exit 1
fi

# Build the image if it doesn't exist
if [[ "$(docker images -q cw4626-vault-dev 2>/dev/null)" == "" ]]; then
  echo "🔨 Building Docker image..."
  docker-compose build
fi

# Start the container
echo "🚀 Starting development container..."
docker-compose up -d

# Wait a moment for container to be ready
sleep 3

# Show container status
echo "📊 Container status:"
docker-compose ps

echo ""
echo "✅ Docker-based Nix environment is ready!"
echo ""
echo "🔧 To enter the container:"
echo "   docker-compose exec cw4626-nix bash"
echo ""
echo "🔧 To run commands directly:"
echo "   docker-compose exec cw4626-nix cargo build"
echo ""
echo "🔧 To stop the environment:"
echo "   docker-compose down"
echo ""
echo "🔧 To view logs:"
echo "   docker-compose logs -f"
echo ""
echo "🔧 To rebuild and restart:"
echo "   docker-compose up -d --build"
echo ""

# Check if user wants to enter the container
read -p "Would you like to enter the container now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  echo "🐳 Entering container..."
  docker-compose exec cw4626-nix nix develop .# --command bash
fi
