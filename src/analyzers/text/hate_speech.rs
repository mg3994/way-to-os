use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Hate speech analyzer for detecting hate speech patterns
pub struct HateSpeechAnalyzer {
    config: TextAnalysisConfig,
    hate_speech_patterns: Vec<(Regex, f64)>,
    targeted_groups: HashMap<String, f64>,
    slurs_and_derogatory: HashMap<String, f64>,
    hate_indicators: HashMap<String, f64>,
}

impl HateSpeechAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            hate_speech_patterns: Vec::new(),
            targeted_groups: HashMap::new(),
            slurs_and_derogatory: HashMap::new(),
            hate_indicators: HashMap::new(),
        };
        
        analyzer.load_hate_speech_patterns();
        analyzer.load_targeted_groups();
        analyzer.load_slurs_and_derogatory();
        analyzer.load_hate_indicators();
        analyzer
    }

    fn load_hate_speech_patterns(&mut self) {
        let patterns = vec![
            // Direct hate speech patterns
            (r"\b(hate|despise|loathe)\s+(all\s+)?(these|those|the)\s+\w+", 0.9),
            (r"\b(all\s+)?\w+\s+(are|should\s+be)\s+(killed|eliminated|removed)", 0.95),
            (r"\b(death\s+to|kill\s+all)\s+\w+", 0.95),
            
            // Supremacist language
            (r"\b(superior|better\s+than|above)\s+(all\s+)?(these|those|the)\s+\w+", 0.7),
            (r"\b(pure|clean)\s+(race|blood|nation)", 0.8),
            (r"\b(master\s+race|chosen\s+people)", 0.9),
            
            // Dehumanizing language
            (r"\b(animals|beasts|vermin|parasites|cockroaches)\b", 0.6),
            (r"\b(subhuman|inhuman|not\s+human)", 0.8),
            (r"\b(disease|plague|infection|virus)\b.*\b(society|world|country)", 0.7),
            
            // Exclusion and segregation
            (r"\b(go\s+back\s+to|return\s+to)\s+\w+", 0.6),
            (r"\b(don't\s+belong|not\s+welcome)\s+(here|in)", 0.7),
            (r"\b(separate|segregate|divide)\s+\w+", 0.5),
            
            // Violence incitement
            (r"\b(should\s+be\s+)?(shot|hanged|burned|beaten)", 0.8),
            (r"\b(time\s+to\s+)?(fight|war|battle)\s+(against|with)\s+\w+", 0.7),
            (r"\b(rise\s+up|take\s+action)\s+against\s+\w+", 0.6),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.hate_speech_patterns.push((regex, weight));
            }
        }
    }

    fn load_targeted_groups(&mut self) {
        // Note: Using general terms to avoid explicit slurs while still detecting hate speech patterns
        let groups = vec![
            // Religious groups
            ("muslim", 0.3), ("christian", 0.3), ("jewish", 0.3), ("hindu", 0.3),
            ("buddhist", 0.3), ("atheist", 0.3), ("religious", 0.2),
            
            // Ethnic/racial (general terms)
            ("immigrant", 0.4), ("refugee", 0.4), ("foreigner", 0.3),
            ("minority", 0.2), ("ethnic", 0.2),
            
            // Gender/sexuality
            ("women", 0.2), ("men", 0.2), ("gay", 0.3), ("lesbian", 0.3),
            ("transgender", 0.3), ("lgbt", 0.3),
            
            // Nationality (when used in derogatory context)
            ("american", 0.2), ("european", 0.2), ("asian", 0.2),
            ("african", 0.2), ("mexican", 0.2),
            
            // Other groups
            ("disabled", 0.3), ("elderly", 0.2), ("poor", 0.2),
            ("homeless", 0.3), ("liberal", 0.2), ("conservative", 0.2),
        ];

        for (group, weight) in groups {
            self.targeted_groups.insert(group.to_string(), weight);
        }
    }

    fn load_slurs_and_derogatory(&mut self) {
        // Mild derogatory terms (avoiding explicit slurs)
        let derogatory = vec![
            ("freak", 0.6), ("weirdo", 0.5), ("creep", 0.6), ("sicko", 0.7),
            ("pervert", 0.7), ("deviant", 0.6), ("degenerate", 0.8),
            ("savage", 0.7), ("primitive", 0.6), ("backward", 0.5),
            ("inferior", 0.7), ("substandard", 0.5), ("lowlife", 0.8),
            ("scumbag", 0.7), ("filth", 0.6), ("dirt", 0.5),
        ];

        for (term, weight) in derogatory {
            self.slurs_and_derogatory.insert(term.to_string(), weight);
        }
    }

    fn load_hate_indicators(&mut self) {
        let indicators = vec![
            // Hate indicators
            ("hate", 0.6), ("despise", 0.7), ("loathe", 0.8), ("detest", 0.7),
            ("disgust", 0.5), ("repulsive", 0.6), ("revolting", 0.6),
            
            // Supremacist indicators
            ("superior", 0.5), ("supreme", 0.6), ("master", 0.6),
            ("pure", 0.4), ("clean", 0.3), ("chosen", 0.5),
            
            // Violence indicators
            ("eliminate", 0.8), ("exterminate", 0.9), ("cleanse", 0.7),
            ("purge", 0.8), ("eradicate", 0.8), ("destroy", 0.6),
            
            // Exclusion indicators
            ("exclude", 0.5), ("ban", 0.4), ("prohibit", 0.4),
            ("forbid", 0.4), ("expel", 0.6), ("deport", 0.6),
        ];

        for (indicator, weight) in indicators {
            self.hate_indicators.insert(indicator.to_string(), weight);
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.hate_speech_patterns.len() + 
                           self.targeted_groups.len() + 
                           self.slurs_and_derogatory.len() + 
                           self.hate_indicators.len();

        // Check for hate speech patterns
        for (pattern, weight) in &self.hate_speech_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("hate_pattern: {}", pattern.as_str()));
            }
        }

        // Check for targeted groups in context
        let group_context_score = self.analyze_group_targeting(&words, &mut detected_patterns);
        if group_context_score > 0.0 {
            total_score += group_context_score;
            matches += 1;
        }

        // Check for slurs and derogatory terms
        for word in &words {
            if let Some(&weight) = self.slurs_and_derogatory.get(*word) {
                let context_multiplier = self.get_context_multiplier(&words, word);
                let adjusted_score = weight * context_multiplier;
                total_score += adjusted_score;
                matches += 1;
                detected_patterns.push(format!("derogatory_term: {}", word));
            }
        }

        // Check for hate indicators
        let mut hate_indicator_score = 0.0;
        for word in &words {
            if let Some(&weight) = self.hate_indicators.get(*word) {
                hate_indicator_score += weight;
                detected_patterns.push(format!("hate_indicator: {}", word));
            }
        }
        
        if hate_indicator_score > 0.0 {
            total_score += (hate_indicator_score / words.len() as f64).min(0.6);
            matches += 1;
        }

        // Cultural and linguistic pattern matching
        let cultural_score = self.analyze_cultural_patterns(&normalized_content, &mut detected_patterns);
        if cultural_score > 0.0 {
            total_score += cultural_score;
            matches += 1;
        }

        // Normalize final score
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / (word_count / 5.0).max(1.0)).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence with context awareness
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

    fn analyze_group_targeting(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut targeting_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&group_weight) = self.targeted_groups.get(*word) {
                // Check surrounding context for hate indicators
                let context_start = if i >= 3 { i - 3 } else { 0 };
                let context_end = std::cmp::min(i + 4, words.len());
                
                let mut context_hate_score = 0.0;
                for j in context_start..context_end {
                    if j != i {
                        if let Some(&hate_weight) = self.hate_indicators.get(words[j]) {
                            context_hate_score += hate_weight;
                        }
                        
                        // Check for negative descriptors
                        let negative_descriptors = ["all", "these", "those", "dirty", "evil", "bad"];
                        if negative_descriptors.contains(&words[j]) {
                            context_hate_score += 0.3;
                        }
                    }
                }
                
                if context_hate_score > 0.0 {
                    let combined_score = group_weight * (1.0 + context_hate_score);
                    targeting_score += combined_score.min(0.9);
                    detected_patterns.push(format!("targeted_group: {} (context_score: {:.2})", word, context_hate_score));
                }
            }
        }
        
        targeting_score.min(0.8)
    }

    fn get_context_multiplier(&self, words: &[&str], target_word: &str) -> f64 {
        let target_index = words.iter().position(|&w| w == target_word);
        if let Some(index) = target_index {
            let context_start = if index >= 2 { index - 2 } else { 0 };
            let context_end = std::cmp::min(index + 3, words.len());
            
            let mut multiplier: f64 = 1.0;
            
            for i in context_start..context_end {
                if i != index {
                    // Intensifiers increase the score
                    let intensifiers = ["very", "extremely", "totally", "completely", "absolutely"];
                    if intensifiers.contains(&words[i]) {
                        multiplier *= 1.3;
                    }
                    
                    // Diminishers decrease the score
                    let diminishers = ["not", "barely", "hardly", "somewhat", "kinda"];
                    if diminishers.contains(&words[i]) {
                        multiplier *= 0.5;
                    }
                }
            }
            
            multiplier.min(2.0).max(0.1)
        } else {
            1.0
        }
    }

    fn analyze_cultural_patterns(&self, content: &str, detected_patterns: &mut Vec<String>) -> f64 {
        let mut cultural_score: f64 = 0.0;
        
        // Check for coded language and dog whistles (general patterns)
        let coded_patterns = vec![
            (r"\b(urban\s+youth|inner\s+city)", 0.3),
            (r"\b(traditional\s+values|real\s+americans)", 0.2),
            (r"\b(globalist|elite|establishment)", 0.2),
            (r"\b(invasion|replacement|takeover)", 0.6),
            (r"\b(purity|bloodline|heritage)", 0.5),
        ];
        
        for (pattern, weight) in coded_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(content) {
                    cultural_score += weight;
                    detected_patterns.push(format!("coded_language: {}", pattern));
                }
            }
        }
        
        cultural_score.min(0.6)
    }

    fn calculate_context_score(&self, words: &[&str]) -> f64 {
        let mut context_score: f64 = 0.5;
        
        // Check for educational or discussion context
        let educational_indicators = ["history", "study", "research", "discuss", "analyze", "understand"];
        let educational_count = words.iter()
            .filter(|word| educational_indicators.contains(word))
            .count();
            
        if educational_count > 0 {
            context_score *= 0.7; // Reduce confidence in educational contexts
        }
        
        // Check for quotation or reporting context
        let reporting_indicators = ["said", "claimed", "reported", "according", "quote"];
        let reporting_count = words.iter()
            .filter(|word| reporting_indicators.contains(word))
            .count();
            
        if reporting_count > 0 {
            context_score *= 0.8; // Reduce confidence when reporting
        }
        
        // Check for negation that might indicate opposition to hate speech
        let opposition_indicators = ["against", "oppose", "condemn", "reject", "fight"];
        let opposition_count = words.iter()
            .filter(|word| opposition_indicators.contains(word))
            .count();
            
        if opposition_count > 0 {
            context_score *= 0.6; // Significantly reduce confidence when opposing hate
        }
        
        context_score.max(0.1).min(1.0)
    }
}