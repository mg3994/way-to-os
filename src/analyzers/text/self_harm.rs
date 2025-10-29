use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Self-harm analyzer for detecting self-harm indicators
pub struct SelfHarmAnalyzer {
    config: TextAnalysisConfig,
    crisis_patterns: Vec<(Regex, f64)>,
    self_harm_keywords: HashMap<String, f64>,
    suicidal_indicators: HashMap<String, f64>,
    emotional_distress_words: HashMap<String, f64>,
}

impl SelfHarmAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            crisis_patterns: Vec::new(),
            self_harm_keywords: HashMap::new(),
            suicidal_indicators: HashMap::new(),
            emotional_distress_words: HashMap::new(),
        };
        
        analyzer.load_crisis_patterns();
        analyzer.load_self_harm_keywords();
        analyzer.load_suicidal_indicators();
        analyzer.load_emotional_distress_words();
        analyzer
    }

    fn load_crisis_patterns(&mut self) {
        let patterns = vec![
            // Direct suicidal ideation
            (r"\bi\s+(want\s+to|am\s+going\s+to|will)\s+(kill\s+myself|end\s+it|die)", 0.95),
            (r"\bkill\s+myself", 0.95),
            (r"\bend\s+(it\s+all|my\s+life)", 0.9),
            (r"\bi\s+(should|would\s+be\s+better\s+off)\s+dead", 0.9),
            
            // Self-harm expressions
            (r"\bi\s+(cut|hurt|harm)\s+myself", 0.8),
            (r"\bi\s+(want\s+to|need\s+to)\s+(cut|hurt|harm)", 0.7),
            (r"\bself\s+(harm|injury|mutilation)", 0.8),
            
            // Hopelessness and despair
            (r"\bno\s+(point|reason|hope)\s+(in\s+)?(living|going\s+on)", 0.8),
            (r"\blife\s+is\s+(not\s+worth|meaningless|pointless)", 0.7),
            (r"\bi\s+can't\s+(take\s+it|go\s+on|handle\s+this)", 0.6),
            
            // Planning indicators
            (r"\bi\s+have\s+a\s+(plan|way|method)", 0.9),
            (r"\bi\s+(know\s+how|figured\s+out\s+how)\s+to", 0.8),
            (r"\btonight\s+is\s+the\s+night", 0.9),
            
            // Goodbye messages
            (r"\bthis\s+is\s+(goodbye|the\s+end)", 0.9),
            (r"\bi\s+won't\s+be\s+(here|around)\s+(much\s+longer|tomorrow)", 0.8),
            (r"\btell\s+(everyone|my\s+family)\s+i\s+(love|loved)", 0.8),
            
            // Burden expressions
            (r"\bi\s+am\s+a\s+(burden|waste|mistake)", 0.7),
            (r"\beveryone\s+would\s+be\s+better\s+off\s+without\s+me", 0.8),
            (r"\bi\s+am\s+(worthless|useless|a\s+failure)", 0.6),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.crisis_patterns.push((regex, weight));
            }
        }
    }

    fn load_self_harm_keywords(&mut self) {
        // High risk self-harm keywords
        let high_risk = vec![
            ("suicide", 0.9), ("suicidal", 0.9), ("kill", 0.7), ("death", 0.5),
            ("die", 0.6), ("dying", 0.6), ("dead", 0.5), ("overdose", 0.8),
            ("pills", 0.4), ("razor", 0.7), ("blade", 0.6), ("cut", 0.5),
            ("cutting", 0.7), ("burn", 0.4), ("burning", 0.5), ("poison", 0.7),
        ];
        
        // Medium risk keywords
        let medium_risk = vec![
            ("hurt", 0.4), ("pain", 0.3), ("suffering", 0.4), ("agony", 0.5),
            ("torture", 0.5), ("misery", 0.4), ("despair", 0.6), ("hopeless", 0.6),
            ("helpless", 0.5), ("trapped", 0.5), ("stuck", 0.3), ("escape", 0.4),
        ];
        
        // Self-referential harm
        let self_referential = vec![
            ("myself", 0.3), ("self", 0.2), ("me", 0.1), ("i", 0.1),
        ];

        for (word, weight) in high_risk.into_iter()
            .chain(medium_risk)
            .chain(self_referential) {
            self.self_harm_keywords.insert(word.to_string(), weight);
        }
    }

    fn load_suicidal_indicators(&mut self) {
        let indicators = vec![
            // Direct indicators
            ("suicide", 0.9), ("suicidal", 0.9), ("kill", 0.6), ("end", 0.4),
            ("finish", 0.3), ("over", 0.2), ("done", 0.3), ("quit", 0.3),
            
            // Method-related
            ("jump", 0.5), ("hang", 0.6), ("rope", 0.5), ("gun", 0.4),
            ("bridge", 0.3), ("building", 0.2), ("train", 0.3), ("car", 0.2),
            
            // Emotional states
            ("hopeless", 0.7), ("worthless", 0.6), ("useless", 0.6),
            ("failure", 0.5), ("burden", 0.6), ("mistake", 0.5),
            ("alone", 0.4), ("isolated", 0.5), ("abandoned", 0.6),
            
            // Finality expressions
            ("forever", 0.4), ("never", 0.3), ("always", 0.2),
            ("final", 0.5), ("last", 0.4), ("goodbye", 0.6),
        ];

        for (word, weight) in indicators {
            self.suicidal_indicators.insert(word.to_string(), weight);
        }
    }

    fn load_emotional_distress_words(&mut self) {
        let distress_words = vec![
            // Severe distress
            ("devastated", 0.7), ("destroyed", 0.6), ("shattered", 0.7),
            ("broken", 0.5), ("crushed", 0.6), ("ruined", 0.5),
            
            // Emotional pain
            ("anguish", 0.6), ("torment", 0.7), ("agony", 0.6),
            ("suffering", 0.5), ("pain", 0.4), ("hurt", 0.4),
            
            // Desperation
            ("desperate", 0.6), ("frantic", 0.5), ("panicked", 0.5),
            ("terrified", 0.5), ("scared", 0.3), ("afraid", 0.3),
            
            // Emptiness
            ("empty", 0.5), ("void", 0.6), ("hollow", 0.5),
            ("numb", 0.5), ("nothing", 0.3), ("meaningless", 0.6),
        ];

        for (word, weight) in distress_words {
            self.emotional_distress_words.insert(word.to_string(), weight);
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.crisis_patterns.len() + 
                           self.self_harm_keywords.len() + 
                           self.suicidal_indicators.len() + 
                           self.emotional_distress_words.len();

        // Check for crisis patterns (highest priority)
        for (pattern, weight) in &self.crisis_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("crisis_pattern: {}", pattern.as_str()));
            }
        }

        // Analyze self-harm keywords with context
        let self_harm_score = self.analyze_self_harm_context(&words, &mut detected_patterns);
        if self_harm_score > 0.0 {
            total_score += self_harm_score;
            matches += 1;
        }

        // Check for suicidal indicators with intent analysis
        let suicidal_score = self.analyze_suicidal_intent(&words, &mut detected_patterns);
        if suicidal_score > 0.0 {
            total_score += suicidal_score;
            matches += 1;
        }

        // Analyze emotional distress level
        let distress_score = self.analyze_emotional_distress(&words, &mut detected_patterns);
        if distress_score > 0.0 {
            total_score += distress_score;
            matches += 1;
        }

        // Check for severity and directness
        let severity_score = self.analyze_severity(&normalized_content, &mut detected_patterns);
        if severity_score > 0.0 {
            total_score += severity_score;
            matches += 1;
        }

        // Analyze temporal urgency
        let urgency_score = self.analyze_urgency(&words, &mut detected_patterns);
        if urgency_score > 0.0 {
            total_score += urgency_score;
            matches += 1;
        }

        // Normalize final score (self-harm detection is more sensitive)
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / (word_count / 12.0).max(1.0)).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence with high sensitivity for false negatives
        let context_score = if self.config.enable_context_analysis {
            self.calculate_context_score(&words)
        } else {
            0.7
        };

        let confidence = calculate_confidence(matches, total_patterns, context_score);

        Ok(AnalysisResult {
            score: normalized_score,
            confidence,
            patterns: detected_patterns,
        })
    }

    fn analyze_self_harm_context(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut harm_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_weight) = self.self_harm_keywords.get(*word) {
                let mut adjusted_weight = base_weight;
                
                // Check for first person context (I, me, my, myself)
                let context_start = if i >= 3 { i - 3 } else { 0 };
                let context_end = std::cmp::min(i + 4, words.len());
                
                let mut is_self_directed = false;
                for j in context_start..context_end {
                    if j != i {
                        let self_pronouns = ["i", "me", "my", "myself"];
                        if self_pronouns.contains(&words[j]) {
                            is_self_directed = true;
                            adjusted_weight *= 2.0; // Significantly increase for self-directed harm
                            break;
                        }
                    }
                }
                
                // Check for intent indicators (want to, going to, will, etc.)
                let mut has_intent = false;
                for j in context_start..context_end {
                    if j != i {
                        let intent_words = ["want", "going", "will", "need", "have", "plan"];
                        if intent_words.contains(&words[j]) {
                            has_intent = true;
                            adjusted_weight *= 1.5;
                            break;
                        }
                    }
                }
                
                harm_score += adjusted_weight.min(1.0);
                
                if is_self_directed && has_intent {
                    detected_patterns.push(format!("self_harm_intent: {}", word));
                } else if is_self_directed {
                    detected_patterns.push(format!("self_directed_harm: {}", word));
                } else {
                    detected_patterns.push(format!("harm_keyword: {}", word));
                }
            }
        }
        
        harm_score.min(0.9)
    }

    fn analyze_suicidal_intent(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut suicidal_score = 0.0;
        let mut indicator_count = 0;
        
        for word in words {
            if let Some(&weight) = self.suicidal_indicators.get(*word) {
                suicidal_score += weight;
                indicator_count += 1;
                detected_patterns.push(format!("suicidal_indicator: {}", word));
            }
        }
        
        // Multiple indicators increase the severity
        if indicator_count > 2 {
            suicidal_score *= 1.3;
            detected_patterns.push(format!("multiple_suicidal_indicators: {}", indicator_count));
        }
        
        (suicidal_score / words.len() as f64 * 5.0).min(0.8)
    }

    fn analyze_emotional_distress(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut distress_score = 0.0;
        let mut distress_count = 0;
        
        for word in words {
            if let Some(&weight) = self.emotional_distress_words.get(*word) {
                distress_score += weight;
                distress_count += 1;
                detected_patterns.push(format!("emotional_distress: {}", word));
            }
        }
        
        if distress_count > 1 {
            distress_score *= 1.2;
            detected_patterns.push(format!("high_emotional_distress: {} indicators", distress_count));
        }
        
        (distress_score / words.len() as f64 * 3.0).min(0.6)
    }

    fn analyze_severity(&self, content: &str, detected_patterns: &mut Vec<String>) -> f64 {
        let mut severity_score: f64 = 0.0;
        
        // Check for severity indicators
        let severity_patterns = vec![
            (r"\btonight\b", 0.8),
            (r"\bright\s+now\b", 0.7),
            (r"\bcan't\s+wait\b", 0.6),
            (r"\bfinal\s+(decision|choice)\b", 0.8),
            (r"\bno\s+turning\s+back\b", 0.7),
            (r"\bmade\s+up\s+my\s+mind\b", 0.7),
        ];
        
        for (pattern, weight) in severity_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(content) {
                    severity_score += weight;
                    detected_patterns.push(format!("severity_indicator: {}", pattern));
                }
            }
        }
        
        severity_score.min(0.8)
    }

    fn analyze_urgency(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut urgency_score = 0.0;
        
        // Time-sensitive words
        let urgent_words = ["now", "tonight", "today", "soon", "immediately", "right"];
        let urgent_count = words.iter()
            .filter(|word| urgent_words.contains(word))
            .count();
        
        if urgent_count > 0 {
            urgency_score += (urgent_count as f64 * 0.2).min(0.6);
            detected_patterns.push(format!("temporal_urgency: {} indicators", urgent_count));
        }
        
        urgency_score
    }

    fn calculate_context_score(&self, words: &[&str]) -> f64 {
        let mut context_score: f64 = 0.7;
        
        // Check for help-seeking context (positive indicator)
        let help_seeking = ["help", "support", "therapy", "counseling", "doctor", "professional"];
        let help_count = words.iter()
            .filter(|word| help_seeking.contains(word))
            .count();
            
        if help_count > 0 {
            context_score *= 1.2; // Increase confidence when help-seeking is mentioned
        }
        
        // Check for past tense (might indicate reflection rather than current intent)
        let past_tense = ["was", "were", "had", "did", "used", "before", "ago"];
        let past_count = words.iter()
            .filter(|word| past_tense.contains(word))
            .count();
            
        if past_count > 1 {
            context_score *= 0.8; // Slightly reduce for past tense
        }
        
        // Check for hypothetical context
        let hypothetical = ["if", "would", "could", "might", "maybe", "perhaps"];
        let hypothetical_count = words.iter()
            .filter(|word| hypothetical.contains(word))
            .count();
            
        if hypothetical_count > 0 {
            context_score *= 0.9; // Slightly reduce for hypothetical
        }
        
        // Check for creative/fictional context
        let fictional = ["story", "character", "book", "movie", "game", "fiction"];
        let fictional_count = words.iter()
            .filter(|word| fictional.contains(word))
            .count();
            
        if fictional_count > 0 {
            context_score *= 0.6; // Significantly reduce for fictional context
        }
        
        context_score.max(0.3).min(1.0)
    }
}