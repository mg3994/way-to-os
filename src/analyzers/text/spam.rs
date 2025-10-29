use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Spam analyzer for detecting spam patterns
pub struct SpamAnalyzer {
    config: TextAnalysisConfig,
    spam_patterns: Vec<(Regex, f64)>,
    promotional_keywords: HashMap<String, f64>,
    url_pattern: Regex,
}

impl SpamAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            spam_patterns: Vec::new(),
            promotional_keywords: HashMap::new(),
            url_pattern: Regex::new(r"https?://[^\s]+|www\.[^\s]+|\b[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b").unwrap(),
        };
        
        analyzer.load_spam_patterns();
        analyzer.load_promotional_keywords();
        analyzer
    }

    fn load_spam_patterns(&mut self) {
        let patterns = vec![
            // Repetitive characters
            (r"(.)\1{4,}", 0.6), // Same character repeated 5+ times
            (r"\b(\w+)\s+\1\s+\1\b", 0.7), // Same word repeated 3 times
            
            // Excessive punctuation
            (r"[!]{5,}", 0.5),
            (r"[?]{5,}", 0.4),
            (r"[.]{5,}", 0.3),
            
            // All caps (excessive)
            (r"\b[A-Z]{10,}\b", 0.6),
            (r"^[A-Z\s!?.,]{20,}$", 0.8), // Entire message in caps
            
            // Common spam phrases
            (r"\b(click\s+here|buy\s+now|act\s+now|limited\s+time)\b", 0.8),
            (r"\b(free\s+money|make\s+money|earn\s+\$|get\s+rich)\b", 0.9),
            (r"\b(no\s+cost|100%\s+free|risk\s+free|guaranteed)\b", 0.7),
            (r"\b(winner|congratulations|you\s+won|claim\s+now)\b", 0.8),
            
            // Contact information spam
            (r"\b(call\s+now|contact\s+us|visit\s+our)\b", 0.6),
            (r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b", 0.5), // Phone numbers
            
            // Cryptocurrency/investment spam
            (r"\b(bitcoin|crypto|investment|trading|forex)\b", 0.4),
            (r"\b(roi|profit|returns|passive\s+income)\b", 0.5),
            
            // Social media spam
            (r"\b(follow\s+me|check\s+out|subscribe|like\s+and\s+share)\b", 0.4),
            (r"\b(dm\s+me|message\s+me|add\s+me)\b", 0.3),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.spam_patterns.push((regex, weight));
            }
        }
    }

    fn load_promotional_keywords(&mut self) {
        // High spam keywords
        let high_spam = vec![
            "sale", "discount", "offer", "deal", "promotion", "coupon",
            "free", "win", "winner", "prize", "lottery", "jackpot",
            "money", "cash", "earn", "profit", "income", "rich",
            "buy", "purchase", "order", "shop", "store", "product"
        ];
        
        // Medium spam keywords
        let medium_spam = vec![
            "click", "visit", "check", "see", "view", "watch",
            "join", "register", "signup", "subscribe", "follow",
            "contact", "call", "email", "message", "text"
        ];
        
        // Low spam keywords
        let low_spam = vec![
            "new", "best", "top", "great", "amazing", "awesome",
            "special", "limited", "exclusive", "premium", "quality"
        ];

        for word in high_spam {
            self.promotional_keywords.insert(word.to_string(), 0.8);
        }
        
        for word in medium_spam {
            self.promotional_keywords.insert(word.to_string(), 0.5);
        }
        
        for word in low_spam {
            self.promotional_keywords.insert(word.to_string(), 0.3);
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.spam_patterns.len() + self.promotional_keywords.len() + 5; // +5 for additional checks

        // Check for spam patterns
        for (pattern, weight) in &self.spam_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("spam_pattern: {}", pattern.as_str()));
            }
        }

        // Check for promotional keywords
        let mut promotional_score = 0.0;
        let mut promotional_count = 0;
        
        for word in &words {
            if let Some(&weight) = self.promotional_keywords.get(*word) {
                promotional_score += weight;
                promotional_count += 1;
                detected_patterns.push(format!("promotional_keyword: {}", word));
            }
        }
        
        if promotional_count > 0 {
            matches += promotional_count;
            // Normalize promotional score by word count but cap it
            total_score += (promotional_score / words.len() as f64 * 2.0).min(0.8);
        }

        // Check for URLs
        if self.url_pattern.is_match(&normalized_content) {
            let url_count = self.url_pattern.find_iter(&normalized_content).count();
            let url_score = (url_count as f64 * 0.3).min(0.7);
            total_score += url_score;
            matches += url_count;
            detected_patterns.push(format!("urls_detected: {}", url_count));
        }

        // Check for repetition analysis
        let repetition_score = self.analyze_repetition(&words);
        if repetition_score > 0.0 {
            total_score += repetition_score;
            matches += 1;
            detected_patterns.push("repetitive_content".to_string());
        }

        // Check caps lock ratio
        let caps_ratio = self.calculate_caps_ratio(content);
        if caps_ratio > 0.5 {
            let caps_score = (caps_ratio - 0.5) * 0.8;
            total_score += caps_score;
            matches += 1;
            detected_patterns.push(format!("excessive_caps: {:.2}", caps_ratio));
        }

        // Check message length and structure
        let structure_score = self.analyze_structure(content, &words);
        if structure_score > 0.0 {
            total_score += structure_score;
            matches += 1;
            detected_patterns.push("suspicious_structure".to_string());
        }

        // Normalize final score
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / (word_count / 10.0).max(1.0)).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence
        let confidence = calculate_confidence(matches, total_patterns, 0.6);

        Ok(AnalysisResult {
            score: normalized_score,
            confidence,
            patterns: detected_patterns,
        })
    }

    fn analyze_repetition(&self, words: &[&str]) -> f64 {
        if words.len() < 3 {
            return 0.0;
        }

        let mut repetition_score = 0.0;
        let mut word_counts = HashMap::new();
        
        // Count word frequencies
        for word in words {
            *word_counts.entry(*word).or_insert(0) += 1;
        }
        
        // Check for excessive repetition
        for (_, count) in word_counts {
            if count > 3 {
                repetition_score += (count as f64 - 3.0) * 0.1;
            }
        }
        
        // Check for consecutive repeated words
        for i in 0..words.len() - 1 {
            if words[i] == words[i + 1] {
                repetition_score += 0.2;
            }
        }
        
        repetition_score.min(0.8)
    }

    fn calculate_caps_ratio(&self, content: &str) -> f64 {
        let total_letters = content.chars().filter(|c| c.is_alphabetic()).count();
        if total_letters == 0 {
            return 0.0;
        }
        
        let caps_letters = content.chars().filter(|c| c.is_uppercase()).count();
        caps_letters as f64 / total_letters as f64
    }

    fn analyze_structure(&self, content: &str, words: &[&str]) -> f64 {
        let mut structure_score = 0.0;
        
        // Very short messages with promotional content
        if words.len() < 5 && content.contains("http") {
            structure_score += 0.4;
        }
        
        // Messages that are mostly numbers (like phone numbers, prices)
        let number_ratio = words.iter()
            .filter(|word| word.chars().any(|c| c.is_numeric()))
            .count() as f64 / words.len() as f64;
            
        if number_ratio > 0.3 {
            structure_score += number_ratio * 0.3;
        }
        
        // Messages with excessive punctuation
        let punct_count = content.chars().filter(|c| c.is_ascii_punctuation()).count();
        let punct_ratio = punct_count as f64 / content.len() as f64;
        
        if punct_ratio > 0.2 {
            structure_score += (punct_ratio - 0.2) * 0.5;
        }
        
        structure_score.min(0.6)
    }
}