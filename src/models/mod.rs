use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Core moderation scores structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModerationScores {
    pub toxicity: f64,
    pub spam: f64,
    pub hate_speech: f64,
    pub harassment: f64,
    pub adult_content: f64,
    pub violence: f64,
    pub self_harm: f64,
    pub overall_risk: f64,
}

impl ModerationScores {
    /// Create a new ModerationScores with all scores set to 0.0
    pub fn new() -> Self {
        Self {
            toxicity: 0.0,
            spam: 0.0,
            hate_speech: 0.0,
            harassment: 0.0,
            adult_content: 0.0,
            violence: 0.0,
            self_harm: 0.0,
            overall_risk: 0.0,
        }
    }

    /// Create ModerationScores from individual scores with validation
    #[allow(dead_code)]
    pub fn from_scores(
        toxicity: f64,
        spam: f64,
        hate_speech: f64,
        harassment: f64,
        adult_content: f64,
        violence: f64,
        self_harm: f64,
        overall_risk: f64,
    ) -> Result<Self, String> {
        let mut scores = Self {
            toxicity,
            spam,
            hate_speech,
            harassment,
            adult_content,
            violence,
            self_harm,
            overall_risk,
        };
        
        scores.validate()?;
        scores.round_scores();
        Ok(scores)
    }

    /// Validate that all scores are within the valid range [0.0, 1.0]
    pub fn validate(&self) -> Result<(), String> {
        let scores = [
            ("toxicity", self.toxicity),
            ("spam", self.spam),
            ("hate_speech", self.hate_speech),
            ("harassment", self.harassment),
            ("adult_content", self.adult_content),
            ("violence", self.violence),
            ("self_harm", self.self_harm),
            ("overall_risk", self.overall_risk),
        ];

        for (name, score) in scores {
            if !(0.0..=1.0).contains(&score) {
                return Err(format!("Score '{}' ({}) is not within valid range [0.0, 1.0]", name, score));
            }
        }

        Ok(())
    }

    /// Round all scores to two decimal places
    pub fn round_scores(&mut self) {
        self.toxicity = (self.toxicity * 100.0).round() / 100.0;
        self.spam = (self.spam * 100.0).round() / 100.0;
        self.hate_speech = (self.hate_speech * 100.0).round() / 100.0;
        self.harassment = (self.harassment * 100.0).round() / 100.0;
        self.adult_content = (self.adult_content * 100.0).round() / 100.0;
        self.violence = (self.violence * 100.0).round() / 100.0;
        self.self_harm = (self.self_harm * 100.0).round() / 100.0;
        self.overall_risk = (self.overall_risk * 100.0).round() / 100.0;
    }

    /// Get the highest scoring category
    #[allow(dead_code)]
    pub fn highest_risk_category(&self) -> (&str, f64) {
        let categories = [
            ("toxicity", self.toxicity),
            ("spam", self.spam),
            ("hate_speech", self.hate_speech),
            ("harassment", self.harassment),
            ("adult_content", self.adult_content),
            ("violence", self.violence),
            ("self_harm", self.self_harm),
        ];

        categories
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, score)| (*name, *score))
            .unwrap_or(("none", 0.0))
    }

    /// Check if any score exceeds the given threshold
    #[allow(dead_code)]
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.toxicity > threshold
            || self.spam > threshold
            || self.hate_speech > threshold
            || self.harassment > threshold
            || self.adult_content > threshold
            || self.violence > threshold
            || self.self_harm > threshold
            || self.overall_risk > threshold
    }

    /// Get all categories that exceed the given threshold
    #[allow(dead_code)]
    pub fn categories_above_threshold(&self, threshold: f64) -> Vec<String> {
        let mut categories = Vec::new();
        
        if self.toxicity > threshold { categories.push("toxicity".to_string()); }
        if self.spam > threshold { categories.push("spam".to_string()); }
        if self.hate_speech > threshold { categories.push("hate_speech".to_string()); }
        if self.harassment > threshold { categories.push("harassment".to_string()); }
        if self.adult_content > threshold { categories.push("adult_content".to_string()); }
        if self.violence > threshold { categories.push("violence".to_string()); }
        if self.self_harm > threshold { categories.push("self_harm".to_string()); }
        if self.overall_risk > threshold { categories.push("overall_risk".to_string()); }
        
        categories
    }
}

impl Default for ModerationScores {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of text analysis
#[derive(Debug, Clone)]
pub struct TextAnalysisResult {
    pub scores: ModerationScores,
    pub detected_patterns: Vec<String>,
    pub confidence: f64,
    pub processing_time: Duration,
}

/// Result of image analysis
#[derive(Debug, Clone)]
pub struct ImageAnalysisResult {
    pub nsfw_score: f64,
    pub skin_percentage: f64,
    pub detected_objects: Vec<String>,
    pub confidence: f64,
    pub processing_time: Duration,
}

/// Result of video analysis
#[derive(Debug, Clone)]
pub struct VideoAnalysisResult {
    pub nsfw_score: f64,
    pub frames_analyzed: usize,
    pub peak_scores: Vec<f64>,
    pub confidence: f64,
    pub processing_time: Duration,
}

/// Request/Response models for API
#[derive(Debug, Deserialize)]
pub struct TextAnalysisRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct TextAnalysisResponse {
    pub scores: ModerationScores,
    pub detected_patterns: Vec<String>,
    pub confidence: f64,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ImageAnalysisResponse {
    pub nsfw_score: f64,
    pub skin_percentage: f64,
    pub detected_objects: Vec<String>,
    pub confidence: f64,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct VideoAnalysisResponse {
    pub nsfw_score: f64,
    pub frames_analyzed: usize,
    pub peak_scores: Vec<f64>,
    pub confidence: f64,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Batch analysis request models
#[derive(Debug, Deserialize)]
pub struct BatchAnalysisRequest {
    pub items: Vec<BatchItem>,
}

#[derive(Debug, Deserialize)]
pub struct BatchItem {
    pub content_type: String, // "text", "image", or "video"
    pub text_content: Option<String>,
    pub binary_content: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct BatchAnalysisResponse {
    pub total_items: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
}

#[derive(Debug, Serialize)]
pub struct BatchItemResult {
    pub index: usize,
    pub success: bool,
    pub text_result: Option<TextAnalysisResponse>,
    pub image_result: Option<ImageAnalysisResponse>,
    pub video_result: Option<VideoAnalysisResponse>,
    pub error: Option<String>,
}