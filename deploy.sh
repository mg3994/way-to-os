#!/bin/bash

# Content Moderation System Deployment Script

set -e

echo "🚀 Starting Content Moderation System deployment..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Build and start the services
echo "🔨 Building Docker image..."
docker-compose build

echo "🏃 Starting services..."
docker-compose up -d

# Wait for the service to be ready
echo "⏳ Waiting for service to be ready..."
sleep 10

# Health check
echo "🏥 Performing health check..."
if curl -f http://localhost:3000/health > /dev/null 2>&1; then
    echo "✅ Content Moderation System is running successfully!"
    echo "📊 Service is available at: http://localhost:3000"
    echo ""
    echo "Available endpoints:"
    echo "  - GET  /health          - Health check"
    echo "  - GET  /metrics         - System metrics"
    echo "  - POST /analyze/text    - Text analysis"
    echo "  - POST /analyze/image   - Image analysis"
    echo "  - POST /analyze/video   - Video analysis"
    echo "  - POST /analyze/batch   - Batch analysis"
    echo ""
    echo "📖 See README.md for API documentation and examples"
else
    echo "❌ Health check failed. Service may not be ready yet."
    echo "📋 Check logs with: docker-compose logs"
    exit 1
fi

echo "🎉 Deployment completed successfully!"