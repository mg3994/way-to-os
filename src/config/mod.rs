use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Main application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub moderation: ModerationConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub max_request_size: usize,
    pub request_timeout_seconds: u64,
}

/// Main moderation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModerationConfig {
    pub thresholds: ThresholdConfig,
    pub text_analysis: TextAnalysisConfig,
    pub image_analysis: ImageAnalysisConfig,
    pub video_analysis: VideoAnalysisConfig,
}

/// Threshold configuration for different content types
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThresholdConfig {
    pub toxicity_threshold: f64,
    pub spam_threshold: f64,
    pub hate_speech_threshold: f64,
    pub harassment_threshold: f64,
    pub adult_content_threshold: f64,
    pub violence_threshold: f64,
    pub self_harm_threshold: f64,
    pub overall_risk_threshold: f64,
}

/// Text analysis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextAnalysisConfig {
    pub enable_context_analysis: bool,
    pub max_text_length: usize,
    pub case_sensitive: bool,
    pub language: String,
}

/// Image analysis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageAnalysisConfig {
    pub max_image_size: usize,
    pub supported_formats: Vec<String>,
    pub skin_detection_threshold: f64,
    pub resize_for_analysis: bool,
    pub max_analysis_width: u32,
    pub max_analysis_height: u32,
}

/// Video analysis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoAnalysisConfig {
    pub max_video_size: usize,
    pub supported_formats: Vec<String>,
    pub frame_sample_interval: u32,
    pub max_frames_to_analyze: usize,
    pub scene_change_threshold: f64,
}

impl AppConfig {
    /// Load configuration from environment variables and default values
    pub fn load() -> Result<Self> {
        let config = Self {
            server: ServerConfig {
                bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
                max_request_size: env::var("MAX_REQUEST_SIZE")
                    .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                    .parse()?,
                request_timeout_seconds: env::var("REQUEST_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
            },
            moderation: ModerationConfig {
                thresholds: ThresholdConfig {
                    toxicity_threshold: env::var("TOXICITY_THRESHOLD")
                        .unwrap_or_else(|_| "0.7".to_string())
                        .parse()?,
                    spam_threshold: env::var("SPAM_THRESHOLD")
                        .unwrap_or_else(|_| "0.8".to_string())
                        .parse()?,
                    hate_speech_threshold: env::var("HATE_SPEECH_THRESHOLD")
                        .unwrap_or_else(|_| "0.6".to_string())
                        .parse()?,
                    harassment_threshold: env::var("HARASSMENT_THRESHOLD")
                        .unwrap_or_else(|_| "0.7".to_string())
                        .parse()?,
                    adult_content_threshold: env::var("ADULT_CONTENT_THRESHOLD")
                        .unwrap_or_else(|_| "0.8".to_string())
                        .parse()?,
                    violence_threshold: env::var("VIOLENCE_THRESHOLD")
                        .unwrap_or_else(|_| "0.7".to_string())
                        .parse()?,
                    self_harm_threshold: env::var("SELF_HARM_THRESHOLD")
                        .unwrap_or_else(|_| "0.7".to_string())
                        .parse()?,
                    overall_risk_threshold: env::var("OVERALL_RISK_THRESHOLD")
                        .unwrap_or_else(|_| "0.6".to_string())
                        .parse()?,
                },
                text_analysis: TextAnalysisConfig {
                    enable_context_analysis: env::var("ENABLE_CONTEXT_ANALYSIS")
                        .unwrap_or_else(|_| "true".to_string())
                        .parse()?,
                    max_text_length: env::var("MAX_TEXT_LENGTH")
                        .unwrap_or_else(|_| "10000".to_string())
                        .parse()?,
                    case_sensitive: env::var("CASE_SENSITIVE")
                        .unwrap_or_else(|_| "false".to_string())
                        .parse()?,
                    language: env::var("LANGUAGE")
                        .unwrap_or_else(|_| "en".to_string()),
                },
                image_analysis: ImageAnalysisConfig {
                    max_image_size: env::var("MAX_IMAGE_SIZE")
                        .unwrap_or_else(|_| "5242880".to_string()) // 5MB
                        .parse()?,
                    supported_formats: vec![
                        "jpeg".to_string(),
                        "jpg".to_string(),
                        "png".to_string(),
                        "gif".to_string(),
                        "webp".to_string(),
                    ],
                    skin_detection_threshold: env::var("SKIN_DETECTION_THRESHOLD")
                        .unwrap_or_else(|_| "0.3".to_string())
                        .parse()?,
                    resize_for_analysis: env::var("RESIZE_FOR_ANALYSIS")
                        .unwrap_or_else(|_| "true".to_string())
                        .parse()?,
                    max_analysis_width: env::var("MAX_ANALYSIS_WIDTH")
                        .unwrap_or_else(|_| "800".to_string())
                        .parse()?,
                    max_analysis_height: env::var("MAX_ANALYSIS_HEIGHT")
                        .unwrap_or_else(|_| "600".to_string())
                        .parse()?,
                },
                video_analysis: VideoAnalysisConfig {
                    max_video_size: env::var("MAX_VIDEO_SIZE")
                        .unwrap_or_else(|_| "52428800".to_string()) // 50MB
                        .parse()?,
                    supported_formats: vec![
                        "mp4".to_string(),
                        "avi".to_string(),
                        "mov".to_string(),
                    ],
                    frame_sample_interval: env::var("FRAME_SAMPLE_INTERVAL")
                        .unwrap_or_else(|_| "3".to_string()) // Every 3 seconds
                        .parse()?,
                    max_frames_to_analyze: env::var("MAX_FRAMES_TO_ANALYZE")
                        .unwrap_or_else(|_| "20".to_string())
                        .parse()?,
                    scene_change_threshold: env::var("SCENE_CHANGE_THRESHOLD")
                        .unwrap_or_else(|_| "0.3".to_string())
                        .parse()?,
                },
            },
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate thresholds are within [0.0, 1.0]
        let thresholds = [
            ("toxicity_threshold", self.moderation.thresholds.toxicity_threshold),
            ("spam_threshold", self.moderation.thresholds.spam_threshold),
            ("hate_speech_threshold", self.moderation.thresholds.hate_speech_threshold),
            ("harassment_threshold", self.moderation.thresholds.harassment_threshold),
            ("adult_content_threshold", self.moderation.thresholds.adult_content_threshold),
            ("violence_threshold", self.moderation.thresholds.violence_threshold),
            ("self_harm_threshold", self.moderation.thresholds.self_harm_threshold),
            ("overall_risk_threshold", self.moderation.thresholds.overall_risk_threshold),
        ];

        for (name, threshold) in thresholds {
            if !(0.0..=1.0).contains(&threshold) {
                return Err(anyhow::anyhow!("Threshold '{}' ({}) must be between 0.0 and 1.0", name, threshold));
            }
        }

        // Validate other constraints
        if self.server.max_request_size == 0 {
            return Err(anyhow::anyhow!("max_request_size must be greater than 0"));
        }

        if self.server.request_timeout_seconds == 0 {
            return Err(anyhow::anyhow!("request_timeout_seconds must be greater than 0"));
        }

        if self.moderation.text_analysis.max_text_length == 0 {
            return Err(anyhow::anyhow!("max_text_length must be greater than 0"));
        }

        Ok(())
    }
}