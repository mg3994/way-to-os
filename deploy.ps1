# Content Moderation System Deployment Script for Windows

Write-Host "🚀 Starting Content Moderation System deployment..." -ForegroundColor Green

# Check if Docker is installed
try {
    docker --version | Out-Null
} catch {
    Write-Host "❌ Docker is not installed. Please install Docker Desktop first." -ForegroundColor Red
    exit 1
}

# Check if Docker Compose is installed
try {
    docker-compose --version | Out-Null
} catch {
    Write-Host "❌ Docker Compose is not installed. Please install Docker Compose first." -ForegroundColor Red
    exit 1
}

# Build and start the services
Write-Host "🔨 Building Docker image..." -ForegroundColor Yellow
docker-compose build

Write-Host "🏃 Starting services..." -ForegroundColor Yellow
docker-compose up -d

# Wait for the service to be ready
Write-Host "⏳ Waiting for service to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Health check
Write-Host "🏥 Performing health check..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/health" -TimeoutSec 5
    if ($response.status -eq "healthy") {
        Write-Host "✅ Content Moderation System is running successfully!" -ForegroundColor Green
        Write-Host "📊 Service is available at: http://localhost:3000" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Available endpoints:" -ForegroundColor White
        Write-Host "  - GET  /health          - Health check" -ForegroundColor Gray
        Write-Host "  - GET  /metrics         - System metrics" -ForegroundColor Gray
        Write-Host "  - POST /analyze/text    - Text analysis" -ForegroundColor Gray
        Write-Host "  - POST /analyze/image   - Image analysis" -ForegroundColor Gray
        Write-Host "  - POST /analyze/video   - Video analysis" -ForegroundColor Gray
        Write-Host "  - POST /analyze/batch   - Batch analysis" -ForegroundColor Gray
        Write-Host ""
        Write-Host "📖 See README.md for API documentation and examples" -ForegroundColor Cyan
    } else {
        throw "Service not healthy"
    }
} catch {
    Write-Host "❌ Health check failed. Service may not be ready yet." -ForegroundColor Red
    Write-Host "📋 Check logs with: docker-compose logs" -ForegroundColor Yellow
    exit 1
}

Write-Host "🎉 Deployment completed successfully!" -ForegroundColor Green