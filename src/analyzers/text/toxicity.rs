use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Toxicity analyzer for detecting toxic language patterns
pub struct ToxicityAnalyzer {
    config: TextAnalysisConfig,
    toxic_words: HashMap<String, f64>,
    toxic_patterns: Vec<(Regex, f64)>,
    intensifiers: HashMap<String, f64>,
    diminishers: HashMap<String, f64>,
}

impl ToxicityAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            toxic_words: HashMap::new(),
            toxic_patterns: Vec::new(),
            intensifiers: HashMap::new(),
            diminishers: HashMap::new(),
        };
        
        analyzer.load_toxic_words();
        analyzer.load_toxic_patterns();
        analyzer.load_modifiers();
        analyzer
    }

    fn load_toxic_words(&mut self) {
        // High toxicity words (0.8-1.0)
        let high_toxic = vec![
            "hate", "stupid", "idiot", "moron", "dumb", "loser", "pathetic", 
            "worthless", "useless", "garbage", "trash", "scum"
        ];
        
        // Medium toxicity words (0.5-0.7)
        let medium_toxic = vec![
            "annoying", "irritating", "obnoxious", "rude", "mean", "nasty",
            "awful", "terrible", "horrible", "disgusting", "gross"
        ];
        
        // Low toxicity words (0.2-0.4)
        let low_toxic = vec![
            "bad", "wrong", "weird", "strange", "odd", "silly", "dumb"
        ];

        for word in high_toxic {
            self.toxic_words.insert(word.to_string(), 0.9);
        }
        
        for word in medium_toxic {
            self.toxic_words.insert(word.to_string(), 0.6);
        }
        
        for word in low_toxic {
            self.toxic_words.insert(word.to_string(), 0.3);
        }
    }

    fn load_toxic_patterns(&mut self) {
        let patterns = vec![
            // Aggressive patterns
            (r"\b(shut\s+up|go\s+away|get\s+lost)\b", 0.7),
            (r"\b(you\s+suck|you\s+fail|you\s+lose)\b", 0.8),
            (r"\b(kill\s+yourself|die\s+already)\b", 0.95),
            
            // Insult patterns
            (r"\b(you\s+are\s+so\s+\w+)\b", 0.5),
            (r"\b(what\s+a\s+\w+)\b", 0.4),
            (r"\b(such\s+a\s+\w+)\b", 0.4),
            
            // Excessive punctuation (anger indicators)
            (r"[!]{3,}", 0.3),
            (r"[?]{3,}", 0.2),
            
            // All caps (shouting)
            (r"\b[A-Z]{4,}\b", 0.2),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.toxic_patterns.push((regex, weight));
            }
        }
    }

    fn load_modifiers(&mut self) {
        // Words that intensify toxicity
        let intensifiers = vec![
            ("very", 1.2), ("extremely", 1.4), ("really", 1.1), ("so", 1.1),
            ("totally", 1.2), ("absolutely", 1.3), ("completely", 1.3),
            ("fucking", 1.5), ("damn", 1.2), ("bloody", 1.2)
        ];

        // Words that diminish toxicity
        let diminishers = vec![
            ("maybe", 0.8), ("perhaps", 0.8), ("somewhat", 0.7), ("kinda", 0.8),
            ("sort of", 0.7), ("kind of", 0.7), ("a bit", 0.6), ("slightly", 0.6)
        ];

        for (word, multiplier) in intensifiers {
            self.intensifiers.insert(word.to_string(), multiplier);
        }

        for (word, multiplier) in diminishers {
            self.diminishers.insert(word.to_string(), multiplier);
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.toxic_words.len() + self.toxic_patterns.len();

        // Check for toxic words
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_score) = self.toxic_words.get(*word) {
                let mut adjusted_score = base_score;
                
                // Check for intensifiers/diminishers in surrounding context
                let context_start = if i >= 2 { i - 2 } else { 0 };
                let context_end = std::cmp::min(i + 3, words.len());
                
                for j in context_start..context_end {
                    if j != i {
                        if let Some(&multiplier) = self.intensifiers.get(words[j]) {
                            adjusted_score *= multiplier;
                        } else if let Some(&multiplier) = self.diminishers.get(words[j]) {
                            adjusted_score *= multiplier;
                        }
                    }
                }
                
                // Cap the score at 1.0
                adjusted_score = adjusted_score.min(1.0);
                total_score += adjusted_score;
                matches += 1;
                detected_patterns.push(format!("toxic_word: {}", word));
            }
        }

        // Check for toxic patterns
        for (pattern, weight) in &self.toxic_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("toxic_pattern: {}", pattern.as_str()));
            }
        }

        // Calculate final score (normalize by number of words to prevent length bias)
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / word_count).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence based on matches and context
        let context_score = if self.config.enable_context_analysis {
            self.calculate_context_score(&words)
        } else {
            0.5
        };

        let confidence = calculate_confidence(matches, total_patterns, context_score);

        Ok(AnalysisResult {
            score: normalized_score,
            confidence,
            patterns: detected_patterns,
        })
    }

    fn calculate_context_score(&self, words: &[&str]) -> f64 {
        let mut context_score: f64 = 0.5; // Base context score
        
        // Check for negation patterns that might reduce toxicity
        let negations = ["not", "don't", "doesn't", "won't", "can't", "isn't", "aren't"];
        let negation_count = words.iter()
            .filter(|word| negations.contains(word))
            .count();
        
        // Negations might indicate sarcasm or contradiction
        if negation_count > 0 {
            context_score *= 0.8;
        }
        
        // Check for question marks (might indicate confusion rather than toxicity)
        let question_indicators = ["what", "why", "how", "when", "where", "who"];
        let question_count = words.iter()
            .filter(|word| question_indicators.contains(word))
            .count();
            
        if question_count > 0 {
            context_score *= 0.9;
        }
        
        // Check for positive words that might balance toxicity
        let positive_words = ["good", "great", "nice", "awesome", "cool", "thanks", "please"];
        let positive_count = words.iter()
            .filter(|word| positive_words.contains(word))
            .count();
            
        if positive_count > 0 {
            context_score *= 0.7;
        }
        
        context_score.max(0.1).min(1.0)
    }
}