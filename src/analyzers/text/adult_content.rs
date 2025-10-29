use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Adult content analyzer for detecting adult content references
pub struct AdultContentAnalyzer {
    config: TextAnalysisConfig,
    explicit_patterns: Vec<(Regex, f64)>,
    sexual_keywords: HashMap<String, f64>,
    suggestive_phrases: Vec<(Regex, f64)>,
    body_part_references: HashMap<String, f64>,
}

impl AdultContentAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            explicit_patterns: Vec::new(),
            sexual_keywords: HashMap::new(),
            suggestive_phrases: Vec::new(),
            body_part_references: HashMap::new(),
        };
        
        analyzer.load_explicit_patterns();
        analyzer.load_sexual_keywords();
        analyzer.load_suggestive_phrases();
        analyzer.load_body_part_references();
        analyzer
    }

    fn load_explicit_patterns(&mut self) {
        let patterns = vec![
            // Direct sexual requests
            (r"\bsend\s+(me\s+)?(nudes|nude\s+pics|naked\s+photos)", 0.9),
            (r"\bshow\s+me\s+your\s+(body|chest|private)", 0.8),
            (r"\bwant\s+to\s+(sleep\s+with|have\s+sex\s+with)", 0.8),
            
            // Sexual activities (keeping it general)
            (r"\bhook\s+up\s+with", 0.6),
            (r"\bone\s+night\s+stand", 0.7),
            (r"\bfriends\s+with\s+benefits", 0.6),
            
            // Inappropriate requests
            (r"\bwhat\s+are\s+you\s+wearing", 0.5),
            (r"\bare\s+you\s+(alone|in\s+bed)", 0.6),
            (r"\bwanna\s+(cyber|sext)", 0.8),
            
            // Dating app inappropriate
            (r"\blooking\s+for\s+(fun|a\s+good\s+time)", 0.4),
            (r"\bno\s+strings\s+attached", 0.5),
            (r"\bcasual\s+(encounter|hookup)", 0.6),
            
            // Explicit descriptions (general patterns)
            (r"\bhot\s+and\s+(sexy|steamy)", 0.5),
            (r"\bturn\s+me\s+on", 0.6),
            (r"\bmake\s+me\s+(wet|hard)", 0.8),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.explicit_patterns.push((regex, weight));
            }
        }
    }

    fn load_sexual_keywords(&mut self) {
        // High explicit content
        let high_explicit = vec![
            ("sex", 0.6), ("sexual", 0.5), ("sexy", 0.4), ("nude", 0.7),
            ("naked", 0.6), ("porn", 0.8), ("adult", 0.3), ("explicit", 0.5),
        ];
        
        // Medium suggestive content
        let medium_suggestive = vec![
            ("hot", 0.2), ("attractive", 0.1), ("beautiful", 0.1), ("gorgeous", 0.1),
            ("cute", 0.1), ("pretty", 0.1), ("handsome", 0.1), ("seductive", 0.4),
            ("sensual", 0.4), ("erotic", 0.6), ("intimate", 0.3), ("romantic", 0.2),
        ];
        
        // Dating/relationship context
        let dating_context = vec![
            ("dating", 0.2), ("relationship", 0.1), ("boyfriend", 0.1), ("girlfriend", 0.1),
            ("partner", 0.1), ("lover", 0.3), ("crush", 0.1), ("flirt", 0.3),
            ("flirting", 0.3), ("attraction", 0.2), ("desire", 0.3), ("lust", 0.5),
        ];
        
        // Inappropriate terms (keeping it mild)
        let inappropriate = vec![
            ("horny", 0.7), ("aroused", 0.6), ("excited", 0.2), ("pleasure", 0.3),
            ("satisfaction", 0.2), ("fantasy", 0.4), ("dream", 0.1), ("imagine", 0.1),
        ];

        for (word, weight) in high_explicit.into_iter()
            .chain(medium_suggestive)
            .chain(dating_context)
            .chain(inappropriate) {
            self.sexual_keywords.insert(word.to_string(), weight);
        }
    }

    fn load_suggestive_phrases(&mut self) {
        let phrases = vec![
            // Suggestive invitations
            (r"\bcome\s+over\s+(tonight|to\s+my\s+place)", 0.5),
            (r"\bwant\s+to\s+(hang\s+out|meet\s+up)\s+(tonight|alone)", 0.4),
            (r"\bmy\s+place\s+or\s+yours", 0.6),
            
            // Compliments with sexual undertones
            (r"\byou\s+look\s+(so\s+)?(hot|sexy|gorgeous)", 0.4),
            (r"\byou\s+have\s+(a\s+)?(nice|great|amazing)\s+(body|figure)", 0.6),
            (r"\bi\s+love\s+your\s+(body|curves|figure)", 0.7),
            
            // Suggestive questions
            (r"\bwhat\s+size\s+(are\s+you|do\s+you\s+wear)", 0.5),
            (r"\bhave\s+you\s+ever\s+(done|tried)", 0.3),
            (r"\bdo\s+you\s+like\s+it\s+(rough|gentle|slow)", 0.6),
            
            // Age-inappropriate content
            (r"\b(barely|just\s+turned)\s+(18|legal)", 0.8),
            (r"\byoung\s+and\s+(tight|fresh)", 0.9),
            (r"\bschool\s+(girl|boy)\s+(fantasy|outfit)", 0.8),
        ];

        for (pattern, weight) in phrases {
            if let Ok(regex) = Regex::new(pattern) {
                self.suggestive_phrases.push((regex, weight));
            }
        }
    }

    fn load_body_part_references(&mut self) {
        // General body parts (lower weight in normal context)
        let general_body = vec![
            ("body", 0.2), ("chest", 0.3), ("legs", 0.2), ("thighs", 0.3),
            ("hips", 0.3), ("curves", 0.4), ("figure", 0.2), ("physique", 0.2),
        ];
        
        // More sensitive references
        let sensitive_references = vec![
            ("private", 0.5), ("intimate", 0.4), ("personal", 0.2),
            ("assets", 0.3), ("package", 0.4), ("goods", 0.3),
        ];

        for (word, weight) in general_body.into_iter()
            .chain(sensitive_references) {
            self.body_part_references.insert(word.to_string(), weight);
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.explicit_patterns.len() + 
                           self.sexual_keywords.len() + 
                           self.suggestive_phrases.len() + 
                           self.body_part_references.len();

        // Check for explicit patterns
        for (pattern, weight) in &self.explicit_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("explicit_pattern: {}", pattern.as_str()));
            }
        }

        // Check for suggestive phrases
        for (pattern, weight) in &self.suggestive_phrases {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("suggestive_phrase: {}", pattern.as_str()));
            }
        }

        // Analyze sexual keywords with context
        let keyword_score = self.analyze_sexual_keywords(&words, &mut detected_patterns);
        if keyword_score > 0.0 {
            total_score += keyword_score;
            matches += 1;
        }

        // Analyze body part references with context
        let body_score = self.analyze_body_references(&words, &mut detected_patterns);
        if body_score > 0.0 {
            total_score += body_score;
            matches += 1;
        }

        // Check for age-inappropriate content
        let age_score = self.analyze_age_appropriateness(&normalized_content, &mut detected_patterns);
        if age_score > 0.0 {
            total_score += age_score;
            matches += 1;
        }

        // Analyze context for legitimate vs inappropriate usage
        let context_multiplier = self.analyze_context(&words);
        total_score *= context_multiplier;

        // Normalize final score
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / (word_count / 8.0).max(1.0)).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence
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

    fn analyze_sexual_keywords(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut keyword_score = 0.0;
        let mut keyword_count = 0;
        
        for word in words {
            if let Some(&base_weight) = self.sexual_keywords.get(*word) {
                keyword_score += base_weight;
                keyword_count += 1;
                detected_patterns.push(format!("sexual_keyword: {}", word));
            }
        }
        
        // Multiple keywords increase the score
        if keyword_count > 2 {
            keyword_score *= 1.3;
            detected_patterns.push(format!("multiple_sexual_keywords: {}", keyword_count));
        }
        
        (keyword_score / words.len() as f64 * 4.0).min(0.8)
    }

    fn analyze_body_references(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut body_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_weight) = self.body_part_references.get(*word) {
                let mut adjusted_weight = base_weight;
                
                // Check for possessive or descriptive context
                let context_start = if i >= 2 { i - 2 } else { 0 };
                let context_end = std::cmp::min(i + 3, words.len());
                
                let mut has_possessive = false;
                for j in context_start..context_end {
                    if j != i {
                        let possessive = ["your", "my", "her", "his", "their"];
                        if possessive.contains(&words[j]) {
                            has_possessive = true;
                            adjusted_weight *= 1.5;
                            break;
                        }
                    }
                }
                
                // Check for descriptive adjectives
                let mut has_descriptive = false;
                for j in context_start..context_end {
                    if j != i {
                        let descriptive = ["nice", "great", "amazing", "beautiful", "perfect", "hot", "sexy"];
                        if descriptive.contains(&words[j]) {
                            has_descriptive = true;
                            adjusted_weight *= 1.3;
                            break;
                        }
                    }
                }
                
                body_score += adjusted_weight.min(1.0);
                
                if has_possessive && has_descriptive {
                    detected_patterns.push(format!("inappropriate_body_reference: {}", word));
                } else if has_possessive || has_descriptive {
                    detected_patterns.push(format!("suggestive_body_reference: {}", word));
                } else {
                    detected_patterns.push(format!("body_reference: {}", word));
                }
            }
        }
        
        body_score.min(0.7)
    }

    fn analyze_age_appropriateness(&self, content: &str, detected_patterns: &mut Vec<String>) -> f64 {
        let mut age_score: f64 = 0.0;
        
        // Check for age-inappropriate patterns
        let age_patterns = vec![
            (r"\b(barely|just)\s+(18|legal)", 0.8),
            (r"\byoung\s+(and\s+)?(tight|fresh|innocent)", 0.9),
            (r"\bteen\s+(fantasy|dream|desire)", 0.9),
            (r"\bschool\s+(uniform|outfit)\s+(fantasy|fetish)", 0.8),
            (r"\bdaddy\s+(issues|kink)", 0.7),
        ];
        
        for (pattern, weight) in age_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(content) {
                    age_score += weight;
                    detected_patterns.push(format!("age_inappropriate: {}", pattern));
                }
            }
        }
        
        age_score.min(0.9)
    }

    fn analyze_context(&self, words: &[&str]) -> f64 {
        let mut context_multiplier: f64 = 1.0;
        
        // Check for medical/health context
        let medical_context = ["doctor", "medical", "health", "healthcare", "clinic", "hospital"];
        let medical_count = words.iter()
            .filter(|word| medical_context.contains(word))
            .count();
            
        if medical_count > 0 {
            context_multiplier *= 0.3; // Significantly reduce for medical context
        }
        
        // Check for educational context
        let educational_context = ["education", "class", "course", "study", "research", "academic"];
        let educational_count = words.iter()
            .filter(|word| educational_context.contains(word))
            .count();
            
        if educational_count > 0 {
            context_multiplier *= 0.4; // Reduce for educational context
        }
        
        // Check for artistic/creative context
        let artistic_context = ["art", "artistic", "creative", "photography", "modeling", "fashion"];
        let artistic_count = words.iter()
            .filter(|word| artistic_context.contains(word))
            .count();
            
        if artistic_count > 0 {
            context_multiplier *= 0.6; // Reduce for artistic context
        }
        
        // Check for relationship/dating context (legitimate)
        let relationship_context = ["relationship", "dating", "marriage", "wedding", "love"];
        let relationship_count = words.iter()
            .filter(|word| relationship_context.contains(word))
            .count();
            
        if relationship_count > 0 {
            context_multiplier *= 0.8; // Slightly reduce for relationship context
        }
        
        context_multiplier.max(0.2).min(1.0)
    }

    fn calculate_context_score(&self, words: &[&str]) -> f64 {
        let mut context_score: f64 = 0.5;
        
        // Check for professional context
        let professional_indicators = ["work", "professional", "business", "career", "job"];
        let professional_count = words.iter()
            .filter(|word| professional_indicators.contains(word))
            .count();
            
        if professional_count > 0 {
            context_score *= 0.7;
        }
        
        // Check for family context
        let family_indicators = ["family", "parents", "children", "kids", "spouse"];
        let family_count = words.iter()
            .filter(|word| family_indicators.contains(word))
            .count();
            
        if family_count > 0 {
            context_score *= 0.6;
        }
        
        // Check for casual/friendly context
        let friendly_indicators = ["friend", "buddy", "pal", "colleague", "neighbor"];
        let friendly_count = words.iter()
            .filter(|word| friendly_indicators.contains(word))
            .count();
            
        if friendly_count > 0 {
            context_score *= 0.8;
        }
        
        context_score.max(0.2).min(1.0)
    }
}