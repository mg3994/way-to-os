use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Harassment analyzer for detecting harassment patterns
pub struct HarassmentAnalyzer {
    config: TextAnalysisConfig,
    harassment_patterns: Vec<(Regex, f64)>,
    personal_attack_words: HashMap<String, f64>,
    threat_indicators: HashMap<String, f64>,
    intimidation_patterns: Vec<(Regex, f64)>,
}

impl HarassmentAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            harassment_patterns: Vec::new(),
            personal_attack_words: HashMap::new(),
            threat_indicators: HashMap::new(),
            intimidation_patterns: Vec::new(),
        };
        
        analyzer.load_harassment_patterns();
        analyzer.load_personal_attack_words();
        analyzer.load_threat_indicators();
        analyzer.load_intimidation_patterns();
        analyzer
    }

    fn load_harassment_patterns(&mut self) {
        let patterns = vec![
            // Direct personal attacks
            (r"\byou\s+(are|look|seem)\s+(so\s+)?(ugly|fat|stupid|dumb|worthless)", 0.8),
            (r"\byou\s+(should|need\s+to|better)\s+(die|disappear|leave|quit)", 0.9),
            (r"\bno\s+one\s+(likes|wants|cares\s+about)\s+you", 0.7),
            
            // Persistent targeting
            (r"\bi\s+(will|am\s+going\s+to|plan\s+to)\s+(find|track|follow)\s+you", 0.9),
            (r"\bi\s+know\s+(where\s+you|who\s+you|your)", 0.8),
            (r"\byou\s+can't\s+(hide|escape|run)", 0.7),
            
            // Sexual harassment patterns
            (r"\bsend\s+(me\s+)?(pics|photos|pictures)", 0.6),
            (r"\bshow\s+me\s+your", 0.7),
            (r"\bwhat\s+are\s+you\s+wearing", 0.5),
            
            // Cyberbullying patterns
            (r"\beveryone\s+(hates|thinks\s+you)", 0.7),
            (r"\byou\s+have\s+no\s+(friends|life|future)", 0.6),
            (r"\bkill\s+yourself", 0.95),
            
            // Doxxing threats
            (r"\bi\s+(will|am\s+going\s+to)\s+(post|share|publish)\s+your", 0.8),
            (r"\byour\s+(address|phone|number|info)", 0.6),
            (r"\bi\s+have\s+your\s+(info|details|address)", 0.8),
            
            // Workplace/academic harassment
            (r"\byou\s+(don't\s+belong|shouldn't\s+be)\s+(here|in)", 0.6),
            (r"\bi\s+will\s+(report|tell|complain)", 0.4),
            (r"\byou\s+will\s+(regret|pay|suffer)", 0.8),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.harassment_patterns.push((regex, weight));
            }
        }
    }

    fn load_personal_attack_words(&mut self) {
        // High severity personal attacks
        let high_severity = vec![
            ("worthless", 0.9), ("pathetic", 0.8), ("disgusting", 0.8),
            ("repulsive", 0.8), ("revolting", 0.8), ("vile", 0.8),
            ("despicable", 0.9), ("contemptible", 0.8), ("abhorrent", 0.9),
        ];
        
        // Medium severity attacks
        let medium_severity = vec![
            ("ugly", 0.6), ("fat", 0.5), ("skinny", 0.4), ("short", 0.3),
            ("tall", 0.2), ("weird", 0.4), ("strange", 0.4), ("creepy", 0.6),
            ("annoying", 0.5), ("irritating", 0.5), ("obnoxious", 0.6),
        ];
        
        // Appearance-based attacks
        let appearance_attacks = vec![
            ("hideous", 0.7), ("gross", 0.5), ("nasty", 0.6), ("filthy", 0.6),
            ("dirty", 0.4), ("smelly", 0.5), ("stinky", 0.5),
        ];
        
        // Intelligence-based attacks
        let intelligence_attacks = vec![
            ("stupid", 0.7), ("dumb", 0.7), ("idiot", 0.8), ("moron", 0.8),
            ("retarded", 0.9), ("brain-dead", 0.8), ("mindless", 0.6),
        ];

        for (word, weight) in high_severity.into_iter()
            .chain(medium_severity)
            .chain(appearance_attacks)
            .chain(intelligence_attacks) {
            self.personal_attack_words.insert(word.to_string(), weight);
        }
    }

    fn load_threat_indicators(&mut self) {
        let threats = vec![
            // Physical threats
            ("hurt", 0.7), ("harm", 0.8), ("beat", 0.8), ("hit", 0.7),
            ("punch", 0.8), ("kick", 0.7), ("attack", 0.8), ("assault", 0.9),
            
            // Intimidation
            ("threaten", 0.8), ("intimidate", 0.8), ("scare", 0.6),
            ("frighten", 0.6), ("terrorize", 0.9), ("menace", 0.8),
            
            // Consequences
            ("punish", 0.6), ("revenge", 0.7), ("payback", 0.7),
            ("retaliate", 0.7), ("get", 0.3), ("destroy", 0.8),
            
            // Stalking
            ("follow", 0.5), ("track", 0.6), ("hunt", 0.8), ("stalk", 0.9),
            ("watch", 0.4), ("monitor", 0.5), ("spy", 0.6),
        ];

        for (word, weight) in threats {
            self.threat_indicators.insert(word.to_string(), weight);
        }
    }

    fn load_intimidation_patterns(&mut self) {
        let patterns = vec![
            // Power dynamics
            (r"\bi\s+(control|own|rule)\s+you", 0.8),
            (r"\byou\s+(work|answer)\s+for\s+me", 0.7),
            (r"\bi\s+am\s+your\s+(boss|superior|master)", 0.8),
            
            // Isolation tactics
            (r"\bno\s+one\s+will\s+(help|save|protect)\s+you", 0.8),
            (r"\byou\s+are\s+(all\s+)?alone", 0.6),
            (r"\bi\s+will\s+make\s+sure\s+(no\s+one|everyone)", 0.7),
            
            // Reputation threats
            (r"\bi\s+will\s+(ruin|destroy)\s+your\s+(reputation|career|life)", 0.8),
            (r"\beveryone\s+will\s+know", 0.6),
            (r"\bi\s+will\s+tell\s+everyone", 0.6),
            
            // Persistent contact
            (r"\bi\s+will\s+(keep|continue|never\s+stop)", 0.6),
            (r"\byou\s+can't\s+(block|ignore|avoid)\s+me", 0.7),
            (r"\bi\s+will\s+find\s+(another\s+way|you)", 0.8),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.intimidation_patterns.push((regex, weight));
            }
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.harassment_patterns.len() + 
                           self.personal_attack_words.len() + 
                           self.threat_indicators.len() + 
                           self.intimidation_patterns.len();

        // Check for harassment patterns
        for (pattern, weight) in &self.harassment_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("harassment_pattern: {}", pattern.as_str()));
            }
        }

        // Check for intimidation patterns
        for (pattern, weight) in &self.intimidation_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("intimidation_pattern: {}", pattern.as_str()));
            }
        }

        // Check for personal attack words with context
        let personal_attack_score = self.analyze_personal_attacks(&words, &mut detected_patterns);
        if personal_attack_score > 0.0 {
            total_score += personal_attack_score;
            matches += 1;
        }

        // Check for threat indicators with context
        let threat_score = self.analyze_threats(&words, &mut detected_patterns);
        if threat_score > 0.0 {
            total_score += threat_score;
            matches += 1;
        }

        // Analyze targeting and persistence
        let targeting_score = self.analyze_targeting(&words, &mut detected_patterns);
        if targeting_score > 0.0 {
            total_score += targeting_score;
            matches += 1;
        }

        // Check for severity escalation patterns
        let escalation_score = self.analyze_escalation(&normalized_content, &mut detected_patterns);
        if escalation_score > 0.0 {
            total_score += escalation_score;
            matches += 1;
        }

        // Normalize final score
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / (word_count / 8.0).max(1.0)).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence with context awareness
        let context_score = if self.config.enable_context_analysis {
            self.calculate_context_score(&words)
        } else {
            0.6
        };

        let confidence = calculate_confidence(matches, total_patterns, context_score);

        Ok(AnalysisResult {
            score: normalized_score,
            confidence,
            patterns: detected_patterns,
        })
    }

    fn analyze_personal_attacks(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut attack_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_weight) = self.personal_attack_words.get(*word) {
                let mut adjusted_weight = base_weight;
                
                // Check if it's directed at someone (you, your, etc.)
                let context_start = if i >= 3 { i - 3 } else { 0 };
                let context_end = std::cmp::min(i + 4, words.len());
                
                let mut is_directed = false;
                for j in context_start..context_end {
                    if j != i {
                        let targeting_words = ["you", "your", "yourself", "u", "ur"];
                        if targeting_words.contains(&words[j]) {
                            is_directed = true;
                            adjusted_weight *= 1.5; // Increase severity for directed attacks
                            break;
                        }
                    }
                }
                
                // Check for intensifiers
                for j in context_start..context_end {
                    if j != i {
                        let intensifiers = ["so", "very", "extremely", "really", "totally", "absolutely"];
                        if intensifiers.contains(&words[j]) {
                            adjusted_weight *= 1.2;
                        }
                    }
                }
                
                attack_score += adjusted_weight.min(1.0);
                
                if is_directed {
                    detected_patterns.push(format!("directed_personal_attack: {}", word));
                } else {
                    detected_patterns.push(format!("personal_attack: {}", word));
                }
            }
        }
        
        attack_score.min(0.9)
    }

    fn analyze_threats(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut threat_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_weight) = self.threat_indicators.get(*word) {
                let mut adjusted_weight = base_weight;
                
                // Check for threat context (I will, going to, etc.)
                let context_start = if i >= 2 { i - 2 } else { 0 };
                let context_end = std::cmp::min(i + 3, words.len());
                
                let mut has_threat_context = false;
                for j in context_start..context_end {
                    if j != i {
                        let threat_context = ["will", "going", "gonna", "plan", "want", "need"];
                        if threat_context.contains(&words[j]) {
                            has_threat_context = true;
                            adjusted_weight *= 1.4;
                            break;
                        }
                    }
                }
                
                // Check for first person (I, me, my)
                let mut is_first_person = false;
                for j in context_start..context_end {
                    if j != i {
                        let first_person = ["i", "me", "my", "myself"];
                        if first_person.contains(&words[j]) {
                            is_first_person = true;
                            adjusted_weight *= 1.3;
                            break;
                        }
                    }
                }
                
                threat_score += adjusted_weight.min(1.0);
                
                if has_threat_context && is_first_person {
                    detected_patterns.push(format!("direct_threat: {}", word));
                } else if has_threat_context || is_first_person {
                    detected_patterns.push(format!("implied_threat: {}", word));
                } else {
                    detected_patterns.push(format!("threat_indicator: {}", word));
                }
            }
        }
        
        threat_score.min(0.8)
    }

    fn analyze_targeting(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut targeting_score = 0.0;
        
        // Count second person pronouns (indicating direct targeting)
        let second_person = ["you", "your", "yourself", "u", "ur"];
        let targeting_count = words.iter()
            .filter(|word| second_person.contains(word))
            .count();
        
        if targeting_count > 2 {
            targeting_score += (targeting_count as f64 * 0.1).min(0.4);
            detected_patterns.push(format!("excessive_targeting: {} instances", targeting_count));
        }
        
        // Check for possessive targeting (your family, your job, etc.)
        for i in 0..words.len() - 1 {
            if words[i] == "your" {
                let personal_targets = ["family", "job", "career", "life", "future", "reputation", 
                                      "friends", "children", "kids", "parents", "home", "address"];
                if personal_targets.contains(&words[i + 1]) {
                    targeting_score += 0.3;
                    detected_patterns.push(format!("personal_targeting: your {}", words[i + 1]));
                }
            }
        }
        
        targeting_score.min(0.6)
    }

    fn analyze_escalation(&self, content: &str, detected_patterns: &mut Vec<String>) -> f64 {
        let mut escalation_score: f64 = 0.0;
        
        // Check for escalation indicators
        let escalation_patterns = vec![
            (r"\bthis\s+is\s+(your\s+)?(last|final)\s+(warning|chance)", 0.7),
            (r"\bnext\s+time", 0.5),
            (r"\bif\s+you\s+(don't|won't)", 0.6),
            (r"\byou\s+(better|should)\s+(watch|be\s+careful)", 0.7),
            (r"\bi\s+warned\s+you", 0.6),
            (r"\bdon't\s+make\s+me", 0.6),
        ];
        
        for (pattern, weight) in escalation_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(content) {
                    escalation_score += weight;
                    detected_patterns.push(format!("escalation_pattern: {}", pattern));
                }
            }
        }
        
        escalation_score.min(0.7)
    }

    fn calculate_context_score(&self, words: &[&str]) -> f64 {
        let mut context_score: f64 = 0.6;
        
        // Check for defensive context
        let defensive_indicators = ["defending", "protect", "self-defense", "response", "reply"];
        let defensive_count = words.iter()
            .filter(|word| defensive_indicators.contains(word))
            .count();
            
        if defensive_count > 0 {
            context_score *= 0.8;
        }
        
        // Check for reporting context
        let reporting_indicators = ["report", "complain", "authorities", "police", "admin"];
        let reporting_count = words.iter()
            .filter(|word| reporting_indicators.contains(word))
            .count();
            
        if reporting_count > 0 {
            context_score *= 0.7;
        }
        
        // Check for joking context (though this can be used to mask harassment)
        let joking_indicators = ["joke", "kidding", "jk", "lol", "haha", "funny"];
        let joking_count = words.iter()
            .filter(|word| joking_indicators.contains(word))
            .count();
            
        if joking_count > 0 {
            context_score *= 0.9; // Only slight reduction as harassment often disguised as jokes
        }
        
        context_score.max(0.2).min(1.0)
    }
}