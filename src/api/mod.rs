use crate::analyzers::text::TextAnalysisEngine;
use crate::analyzers::image::ImageAnalysisEngine;
use crate::analyzers::video::VideoAnalysisEngine;
use crate::analyzers::{TextAnalyzer, ImageAnalyzer, VideoAnalyzer};
use crate::config::AppConfig;
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub text_analyzer: Arc<TextAnalysisEngine>,
    pub image_analyzer: Arc<ImageAnalysisEngine>,
    pub video_analyzer: Arc<VideoAnalysisEngine>,
    pub start_time: std::time::Instant,
}

/// Create the main application router
pub async fn create_app(config: AppConfig) -> Result<Router> {
    // Initialize analyzers
    let text_analyzer = Arc::new(TextAnalysisEngine::new(config.moderation.text_analysis.clone()));
    let image_analyzer = Arc::new(ImageAnalysisEngine::new(config.moderation.image_analysis.clone()));
    let video_analyzer = Arc::new(VideoAnalysisEngine::new(config.moderation.video_analysis.clone()));

    // Create application state
    let state = AppState {
        config: config.clone(),
        text_analyzer,
        image_analyzer,
        video_analyzer,
        start_time: std::time::Instant::now(),
    };

    // Build the router
    let app = Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        
        // Analysis endpoints
        .route("/analyze/text", post(analyze_text))
        .route("/analyze/image", post(analyze_image))
        .route("/analyze/video", post(analyze_video))
        .route("/analyze/batch", post(analyze_batch))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state);

    Ok(app)
}

/// Health check handler
async fn health_check(State(state): State<AppState>) -> Json<crate::models::HealthResponse> {
    Json(crate::models::HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

/// Metrics handler
async fn metrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "version": env!("CARGO_PKG_VERSION"),
        "config": {
            "max_request_size": state.config.server.max_request_size,
            "request_timeout_seconds": state.config.server.request_timeout_seconds,
        }
    }))
}

/// Text analysis handler
async fn analyze_text(
    State(state): State<AppState>,
    Json(request): Json<crate::models::TextAnalysisRequest>,
) -> Result<Json<crate::models::TextAnalysisResponse>, (StatusCode, Json<crate::models::ErrorResponse>)> {
    // Validate input
    if request.content.is_empty() {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Content cannot be empty".to_string(),
            "EMPTY_CONTENT".to_string(),
        ));
    }

    if request.content.len() > state.config.moderation.text_analysis.max_text_length {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            format!("Content too long. Maximum length is {} characters", 
                   state.config.moderation.text_analysis.max_text_length),
            "CONTENT_TOO_LONG".to_string(),
        ));
    }

    // Analyze the text
    match state.text_analyzer.analyze_text(&request.content).await {
        Ok(result) => {
            let response = crate::models::TextAnalysisResponse {
                scores: result.scores,
                detected_patterns: result.detected_patterns,
                confidence: result.confidence,
                processing_time_ms: result.processing_time.as_millis() as u64,
            };
            Ok(Json(response))
        }
        Err(e) => Err(create_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Analysis failed: {}", e),
            "ANALYSIS_FAILED".to_string(),
        )),
    }
}

/// Image analysis handler
async fn analyze_image(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Json<crate::models::ImageAnalysisResponse>, (StatusCode, Json<crate::models::ErrorResponse>)> {
    // Validate input size
    if body.is_empty() {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Image data cannot be empty".to_string(),
            "EMPTY_IMAGE".to_string(),
        ));
    }

    if body.len() > state.config.moderation.image_analysis.max_image_size {
        return Err(create_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Image too large. Maximum size is {} bytes", 
                   state.config.moderation.image_analysis.max_image_size),
            "IMAGE_TOO_LARGE".to_string(),
        ));
    }

    // Analyze the image
    match state.image_analyzer.analyze_image(&body).await {
        Ok(result) => {
            let response = crate::models::ImageAnalysisResponse {
                nsfw_score: result.nsfw_score,
                skin_percentage: result.skin_percentage,
                detected_objects: result.detected_objects,
                confidence: result.confidence,
                processing_time_ms: result.processing_time.as_millis() as u64,
            };
            Ok(Json(response))
        }
        Err(e) => Err(create_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Image analysis failed: {}", e),
            "ANALYSIS_FAILED".to_string(),
        )),
    }
}

/// Video analysis handler
async fn analyze_video(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Result<Json<crate::models::VideoAnalysisResponse>, (StatusCode, Json<crate::models::ErrorResponse>)> {
    // Validate input size
    if body.is_empty() {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Video data cannot be empty".to_string(),
            "EMPTY_VIDEO".to_string(),
        ));
    }

    if body.len() > state.config.moderation.video_analysis.max_video_size {
        return Err(create_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Video too large. Maximum size is {} bytes", 
                   state.config.moderation.video_analysis.max_video_size),
            "VIDEO_TOO_LARGE".to_string(),
        ));
    }

    // Analyze the video
    match state.video_analyzer.analyze_video(&body).await {
        Ok(result) => {
            let response = crate::models::VideoAnalysisResponse {
                nsfw_score: result.nsfw_score,
                frames_analyzed: result.frames_analyzed,
                peak_scores: result.peak_scores,
                confidence: result.confidence,
                processing_time_ms: result.processing_time.as_millis() as u64,
            };
            Ok(Json(response))
        }
        Err(e) => Err(create_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Video analysis failed: {}", e),
            "ANALYSIS_FAILED".to_string(),
        )),
    }
}

/// Batch analysis handler
async fn analyze_batch(
    State(state): State<AppState>,
    Json(request): Json<crate::models::BatchAnalysisRequest>,
) -> Result<Json<crate::models::BatchAnalysisResponse>, (StatusCode, Json<crate::models::ErrorResponse>)> {
    // Validate input
    if request.items.is_empty() {
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Batch request cannot be empty".to_string(),
            "EMPTY_BATCH".to_string(),
        ));
    }

    if request.items.len() > 100 { // Limit batch size
        return Err(create_error_response(
            StatusCode::BAD_REQUEST,
            "Batch size too large. Maximum 100 items per request".to_string(),
            "BATCH_TOO_LARGE".to_string(),
        ));
    }

    let mut results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    // Process each item in the batch
    for (index, item) in request.items.iter().enumerate() {
        match item.content_type.as_str() {
            "text" => {
                if let Some(content) = &item.text_content {
                    match state.text_analyzer.analyze_text(content).await {
                        Ok(result) => {
                            results.push(crate::models::BatchItemResult {
                                index,
                                success: true,
                                text_result: Some(crate::models::TextAnalysisResponse {
                                    scores: result.scores,
                                    detected_patterns: result.detected_patterns,
                                    confidence: result.confidence,
                                    processing_time_ms: result.processing_time.as_millis() as u64,
                                }),
                                image_result: None,
                                video_result: None,
                                error: None,
                            });
                            successful += 1;
                        }
                        Err(e) => {
                            results.push(crate::models::BatchItemResult {
                                index,
                                success: false,
                                text_result: None,
                                image_result: None,
                                video_result: None,
                                error: Some(format!("Text analysis failed: {}", e)),
                            });
                            failed += 1;
                        }
                    }
                } else {
                    results.push(crate::models::BatchItemResult {
                        index,
                        success: false,
                        text_result: None,
                        image_result: None,
                        video_result: None,
                        error: Some("Missing text content".to_string()),
                    });
                    failed += 1;
                }
            }
            "image" => {
                if let Some(image_data) = &item.binary_content {
                    match state.image_analyzer.analyze_image(image_data).await {
                        Ok(result) => {
                            results.push(crate::models::BatchItemResult {
                                index,
                                success: true,
                                text_result: None,
                                image_result: Some(crate::models::ImageAnalysisResponse {
                                    nsfw_score: result.nsfw_score,
                                    skin_percentage: result.skin_percentage,
                                    detected_objects: result.detected_objects,
                                    confidence: result.confidence,
                                    processing_time_ms: result.processing_time.as_millis() as u64,
                                }),
                                video_result: None,
                                error: None,
                            });
                            successful += 1;
                        }
                        Err(e) => {
                            results.push(crate::models::BatchItemResult {
                                index,
                                success: false,
                                text_result: None,
                                image_result: None,
                                video_result: None,
                                error: Some(format!("Image analysis failed: {}", e)),
                            });
                            failed += 1;
                        }
                    }
                } else {
                    results.push(crate::models::BatchItemResult {
                        index,
                        success: false,
                        text_result: None,
                        image_result: None,
                        video_result: None,
                        error: Some("Missing image data".to_string()),
                    });
                    failed += 1;
                }
            }
            "video" => {
                if let Some(video_data) = &item.binary_content {
                    match state.video_analyzer.analyze_video(video_data).await {
                        Ok(result) => {
                            results.push(crate::models::BatchItemResult {
                                index,
                                success: true,
                                text_result: None,
                                image_result: None,
                                video_result: Some(crate::models::VideoAnalysisResponse {
                                    nsfw_score: result.nsfw_score,
                                    frames_analyzed: result.frames_analyzed,
                                    peak_scores: result.peak_scores,
                                    confidence: result.confidence,
                                    processing_time_ms: result.processing_time.as_millis() as u64,
                                }),
                                error: None,
                            });
                            successful += 1;
                        }
                        Err(e) => {
                            results.push(crate::models::BatchItemResult {
                                index,
                                success: false,
                                text_result: None,
                                image_result: None,
                                video_result: None,
                                error: Some(format!("Video analysis failed: {}", e)),
                            });
                            failed += 1;
                        }
                    }
                } else {
                    results.push(crate::models::BatchItemResult {
                        index,
                        success: false,
                        text_result: None,
                        image_result: None,
                        video_result: None,
                        error: Some("Missing video data".to_string()),
                    });
                    failed += 1;
                }
            }
            _ => {
                results.push(crate::models::BatchItemResult {
                    index,
                    success: false,
                    text_result: None,
                    image_result: None,
                    video_result: None,
                    error: Some(format!("Unsupported content type: {}", item.content_type)),
                });
                failed += 1;
            }
        }
    }

    let response = crate::models::BatchAnalysisResponse {
        total_items: request.items.len(),
        successful,
        failed,
        results,
    };

    Ok(Json(response))
}

/// Helper function to create error responses
fn create_error_response(
    status: StatusCode,
    message: String,
    code: String,
) -> (StatusCode, Json<crate::models::ErrorResponse>) {
    (
        status,
        Json(crate::models::ErrorResponse {
            error: message,
            code,
            timestamp: chrono::Utc::now(),
        }),
    )
}