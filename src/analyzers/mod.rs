pub mod text;
pub mod image;
pub mod video;

use crate::models::{TextAnalysisResult, ImageAnalysisResult, VideoAnalysisResult};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for content analyzers
#[allow(dead_code)]
#[async_trait]
pub trait ContentAnalyzer {
    type Input;
    type Output;

    async fn analyze(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Trait for text content analysis
#[async_trait]
pub trait TextAnalyzer: Send + Sync {
    async fn analyze_text(&self, content: &str) -> Result<TextAnalysisResult>;
}

/// Trait for image content analysis
#[async_trait]
pub trait ImageAnalyzer: Send + Sync {
    async fn analyze_image(&self, image_data: &[u8]) -> Result<ImageAnalysisResult>;
}

/// Trait for video content analysis
#[async_trait]
pub trait VideoAnalyzer: Send + Sync {
    async fn analyze_video(&self, video_data: &[u8]) -> Result<VideoAnalysisResult>;
}