# Content Moderation System

A comprehensive, self-contained Rust-based content moderation system that analyzes chat messages for harmful content and detects NSFW material in images and videos. The system uses rule-based algorithms, pattern matching, and traditional computer vision techniques without requiring external AI services or third-party LLMs.

## Features

- **Text Analysis**: Detects toxicity, spam, hate speech, harassment, self-harm indicators, violence, and adult content
- **Image Analysis**: NSFW content detection using computer vision techniques (coming soon)
- **Video Analysis**: Frame sampling and temporal analysis for video content (coming soon)
- **REST API**: Easy-to-use HTTP endpoints for integration
- **Configurable Thresholds**: Customize detection sensitivity for different use cases
- **Real-time Processing**: Fast analysis with sub-second response times
- **Self-contained**: No external dependencies or API keys required

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Git

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd content-moderation-system
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run
```

The server will start on `http://localhost:3000` by default.

## API Documentation

### Health Check

Check if the service is running:

```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 42
}
```

### System Metrics

Get system metrics and configuration:

```bash
curl http://localhost:3000/metrics
```

**Response:**
```json
{
  "uptime_seconds": 42,
  "version": "0.1.0",
  "config": {
    "max_request_size": 10485760,
    "request_timeout_seconds": 30
  }
}
```

### Text Analysis

Analyze text content for harmful patterns:

```bash
curl -X POST http://localhost:3000/analyze/text \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello world, this is a test message."}'
```

**Response:**
```json
{
  "scores": {
    "toxicity": 0.0,
    "spam": 0.0,
    "hate_speech": 0.0,
    "harassment": 0.0,
    "adult_content": 0.0,
    "violence": 0.0,
    "self_harm": 0.0,
    "overall_risk": 0.0
  },
  "detected_patterns": [],
  "confidence": 0.0,
  "processing_time_ms": 1
}
```

### PowerShell Examples

For Windows users using PowerShell:

```powershell
# Health check
Invoke-RestMethod -Uri "http://localhost:3000/health"

# Text analysis
$body = @{
    content = "Hello world, this is a test message."
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:3000/analyze/text" -Method POST -ContentType "application/json" -Body $body
```

## ModerationScores Structure

The system returns detailed scores for different categories:

| Field | Description | Range |
|-------|-------------|-------|
| `toxicity` | General toxic language detection | 0.0 - 1.0 |
| `spam` | Repetitive or promotional content | 0.0 - 1.0 |
| `hate_speech` | Targeted hate speech | 0.0 - 1.0 |
| `harassment` | Personal attacks and harassment | 0.0 - 1.0 |
| `adult_content` | Sexual or adult content references | 0.0 - 1.0 |
| `violence` | Violence-related content | 0.0 - 1.0 |
| `self_harm` | Self-harm indicators | 0.0 - 1.0 |
| `overall_risk` | Weighted overall risk score | 0.0 - 1.0 |

All scores are rounded to 2 decimal places and represent the confidence level (0.0 = safe, 1.0 = high risk).

## Configuration

The system can be configured using environment variables or a `.env` file:

### Server Configuration
```env
BIND_ADDRESS=0.0.0.0:3000
MAX_REQUEST_SIZE=10485760
REQUEST_TIMEOUT_SECONDS=30
```

### Moderation Thresholds
```env
TOXICITY_THRESHOLD=0.7
SPAM_THRESHOLD=0.8
HATE_SPEECH_THRESHOLD=0.6
HARASSMENT_THRESHOLD=0.7
ADULT_CONTENT_THRESHOLD=0.8
VIOLENCE_THRESHOLD=0.7
SELF_HARM_THRESHOLD=0.7
OVERALL_RISK_THRESHOLD=0.6
```

### Text Analysis Settings
```env
ENABLE_CONTEXT_ANALYSIS=true
MAX_TEXT_LENGTH=10000
CASE_SENSITIVE=false
LANGUAGE=en
```

### Image Analysis Settings
```env
MAX_IMAGE_SIZE=5242880
SKIN_DETECTION_THRESHOLD=0.3
RESIZE_FOR_ANALYSIS=true
MAX_ANALYSIS_WIDTH=800
MAX_ANALYSIS_HEIGHT=600
```

### Video Analysis Settings
```env
MAX_VIDEO_SIZE=52428800
FRAME_SAMPLE_INTERVAL=3
MAX_FRAMES_TO_ANALYZE=20
SCENE_CHANGE_THRESHOLD=0.3
```

## Integration Examples

### JavaScript/Node.js

```javascript
const axios = require('axios');

async function analyzeText(content) {
  try {
    const response = await axios.post('http://localhost:3000/analyze/text', {
      content: content
    });
    
    const { scores, detected_patterns, confidence } = response.data;
    
    // Check if content exceeds thresholds
    if (scores.overall_risk > 0.6) {
      console.log('High risk content detected!');
      console.log('Risk categories:', Object.entries(scores)
        .filter(([_, score]) => score > 0.5)
        .map(([category, _]) => category)
      );
    }
    
    return response.data;
  } catch (error) {
    console.error('Analysis failed:', error.message);
  }
}

// Usage
analyzeText("Hello world, this is a test message.")
  .then(result => console.log(result));
```

### Python

```python
import requests
import json

def analyze_text(content):
    url = "http://localhost:3000/analyze/text"
    payload = {"content": content}
    
    try:
        response = requests.post(url, json=payload)
        response.raise_for_status()
        
        data = response.json()
        scores = data['scores']
        
        # Check for high-risk categories
        high_risk_categories = [
            category for category, score in scores.items() 
            if score > 0.5
        ]
        
        if high_risk_categories:
            print(f"High risk categories detected: {high_risk_categories}")
        
        return data
        
    except requests.exceptions.RequestException as e:
        print(f"Analysis failed: {e}")
        return None

# Usage
result = analyze_text("Hello world, this is a test message.")
if result:
    print(json.dumps(result, indent=2))
```

### Rust Client

```rust
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TextAnalysisRequest {
    content: String,
}

#[derive(Deserialize)]
struct ModerationScores {
    toxicity: f64,
    spam: f64,
    hate_speech: f64,
    harassment: f64,
    adult_content: f64,
    violence: f64,
    self_harm: f64,
    overall_risk: f64,
}

#[derive(Deserialize)]
struct TextAnalysisResponse {
    scores: ModerationScores,
    detected_patterns: Vec<String>,
    confidence: f64,
    processing_time_ms: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request = TextAnalysisRequest {
        content: "Hello world, this is a test message.".to_string(),
    };
    
    let response = client
        .post("http://localhost:3000/analyze/text")
        .json(&request)
        .send()
        .await?;
    
    let analysis: TextAnalysisResponse = response.json().await?;
    
    println!("Overall risk: {}", analysis.scores.overall_risk);
    println!("Processing time: {}ms", analysis.processing_time_ms);
    
    if analysis.scores.overall_risk > 0.6 {
        println!("High risk content detected!");
    }
    
    Ok(())
}
```

## Error Handling

The API returns appropriate HTTP status codes and error messages:

### 400 Bad Request
```json
{
  "error": "Content cannot be empty",
  "code": "EMPTY_CONTENT",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### 413 Payload Too Large
```json
{
  "error": "Content too long. Maximum length is 10000 characters",
  "code": "CONTENT_TOO_LONG",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### 500 Internal Server Error
```json
{
  "error": "Analysis failed: Internal processing error",
  "code": "ANALYSIS_FAILED",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### 501 Not Implemented
```json
{
  "error": "Image analysis not yet implemented",
  "code": "NOT_IMPLEMENTED",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Performance

- **Text Analysis**: < 2 seconds per request (as per requirements)
- **Memory Usage**: Optimized for concurrent processing
- **Throughput**: Handles multiple concurrent requests efficiently
- **Scalability**: Stateless design allows horizontal scaling

## Development Status

### ✅ Completed Features
- [x] Project structure and core interfaces
- [x] REST API with health checks and metrics
- [x] Text analysis endpoint (with placeholder algorithms)
- [x] Configuration management
- [x] Error handling and validation
- [x] ModerationScores structure with full validation

### 🚧 In Progress
- [ ] Text analysis algorithms (toxicity, spam, hate speech, etc.)
- [ ] Image analysis with NSFW detection
- [ ] Video analysis with frame sampling
- [ ] Batch processing endpoints

### 📋 Planned Features
- [ ] Advanced pattern matching algorithms
- [ ] Machine learning model integration (optional)
- [ ] Performance monitoring and metrics
- [ ] Docker containerization
- [ ] Comprehensive test suite

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For questions, issues, or contributions, please:
- Open an issue on GitHub
- Check the documentation
- Review the API examples above

## Architecture

The system follows a modular architecture:

```
├── src/
│   ├── analyzers/          # Content analysis engines
│   │   ├── text/           # Text analysis modules
│   │   ├── image/          # Image analysis modules
│   │   └── video/          # Video analysis modules
│   ├── api/                # REST API handlers
│   ├── config/             # Configuration management
│   ├── models/             # Data structures
│   └── utils/              # Utility functions
├── .env                    # Environment configuration
└── Cargo.toml             # Rust dependencies
```

Each analyzer is self-contained and can be developed independently, making the system highly maintainable and extensible.