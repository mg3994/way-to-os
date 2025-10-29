pub mod toxicity;
pub mod spam;
pub mod hate_speech;
pub mod harassment;
pub mod self_harm;
pub mod violence;
pub mod adult_content;

use crate::analyzers::TextAnalyzer;
use crate::config::TextAnalysisConfig;
use crate::models::{ModerationScores, TextAnalysisResult};
use anyhow::Result;
use async_trait::async_trait;
use std::time::Instant;

/// Main text analyzer that combines all text analysis modules
pub struct TextAnalysisEngine {
    #[allow(dead_code)]
    config: TextAnalysisConfig,
    toxicity_analyzer: toxicity::ToxicityAnalyzer,
    spam_analyzer: spam::SpamAnalyzer,
    hate_speech_analyzer: hate_speech::HateSpeechAnalyzer,
    harassment_analyzer: harassment::HarassmentAnalyzer,
    self_harm_analyzer: self_harm::SelfHarmAnalyzer,
    violence_analyzer: violence::ViolenceAnalyzer,
    adult_content_analyzer: adult_content::AdultContentAnalyzer,
}

impl TextAnalysisEngine {
    pub fn new(config: TextAnalysisConfig) -> Self {
        Self {
            toxicity_analyzer: toxicity::ToxicityAnalyzer::new(&config),
            spam_analyzer: spam::SpamAnalyzer::new(&config),
            hate_speech_analyzer: hate_speech::HateSpeechAnalyzer::new(&config),
            harassment_analyzer: harassment::HarassmentAnalyzer::new(&config),
            self_harm_analyzer: self_harm::SelfHarmAnalyzer::new(&config),
            violence_analyzer: violence::ViolenceAnalyzer::new(&config),
            adult_content_analyzer: adult_content::AdultContentAnalyzer::new(&config),
            config,
        }
    }
}

#[async_trait]
impl TextAnalyzer for TextAnalysisEngine {
    async fn analyze_text(&self, content: &str) -> Result<TextAnalysisResult> {
        let start_time = Instant::now();
        let mut detected_patterns = Vec::new();

        // Run all analyzers
        let toxicity_result = self.toxicity_analyzer.analyze(content).await?;
        let spam_result = self.spam_analyzer.analyze(content).await?;
        let hate_speech_result = self.hate_speech_analyzer.analyze(content).await?;
        let harassment_result = self.harassment_analyzer.analyze(content).await?;
        let self_harm_result = self.self_harm_analyzer.analyze(content).await?;
        let violence_result = self.violence_analyzer.analyze(content).await?;
        let adult_content_result = self.adult_content_analyzer.analyze(content).await?;

        // Collect detected patterns
        detected_patterns.extend(toxicity_result.patterns);
        detected_patterns.extend(spam_result.patterns);
        detected_patterns.extend(hate_speech_result.patterns);
        detected_patterns.extend(harassment_result.patterns);
        detected_patterns.extend(self_harm_result.patterns);
        detected_patterns.extend(violence_result.patterns);
        detected_patterns.extend(adult_content_result.patterns);

        // Create moderation scores
        let mut scores = ModerationScores {
            toxicity: toxicity_result.score,
            spam: spam_result.score,
            hate_speech: hate_speech_result.score,
            harassment: harassment_result.score,
            adult_content: adult_content_result.score,
            violence: violence_result.score,
            self_harm: self_harm_result.score,
            overall_risk: 0.0, // Will be calculated below
        };

        // Calculate overall risk score (weighted average)
        scores.overall_risk = calculate_overall_risk(&scores);
        
        // Round scores to two decimal places
        scores.round_scores();

        // Validate scores
        scores.validate().map_err(|e| anyhow::anyhow!("Score validation failed: {}", e))?;

        // Calculate average confidence
        let confidence = (toxicity_result.confidence + spam_result.confidence + 
                         hate_speech_result.confidence + harassment_result.confidence +
                         self_harm_result.confidence + violence_result.confidence +
                         adult_content_result.confidence) / 7.0;

        let processing_time = start_time.elapsed();

        Ok(TextAnalysisResult {
            scores,
            detected_patterns,
            confidence,
            processing_time,
        })
    }
}

/// Calculate overall risk score based on individual scores
fn calculate_overall_risk(scores: &ModerationScores) -> f64 {
    // Weighted calculation - some categories are more critical than others
    let weights = [
        (scores.toxicity, 0.2),
        (scores.spam, 0.1),
        (scores.hate_speech, 0.2),
        (scores.harassment, 0.15),
        (scores.adult_content, 0.1),
        (scores.violence, 0.15),
        (scores.self_harm, 0.1),
    ];

    let weighted_sum: f64 = weights.iter().map(|(score, weight)| score * weight).sum();
    
    // Ensure the result is within [0.0, 1.0]
    weighted_sum.min(1.0).max(0.0)
}

/// Common analysis result structure for individual analyzers
#[derive(Debug)]
pub struct AnalysisResult {
    pub score: f64,
    pub confidence: f64,
    pub patterns: Vec<String>,
}