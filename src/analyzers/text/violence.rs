use crate::analyzers::text::AnalysisResult;
use crate::config::TextAnalysisConfig;
use crate::utils::{normalize_text, calculate_confidence};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Violence analyzer for detecting violence-related content
pub struct ViolenceAnalyzer {
    config: TextAnalysisConfig,
    violence_patterns: Vec<(Regex, f64)>,
    weapon_keywords: HashMap<String, f64>,
    violent_actions: HashMap<String, f64>,
    threat_patterns: Vec<(Regex, f64)>,
}

impl ViolenceAnalyzer {
    pub fn new(config: &TextAnalysisConfig) -> Self {
        let mut analyzer = Self {
            config: config.clone(),
            violence_patterns: Vec::new(),
            weapon_keywords: HashMap::new(),
            violent_actions: HashMap::new(),
            threat_patterns: Vec::new(),
        };
        
        analyzer.load_violence_patterns();
        analyzer.load_weapon_keywords();
        analyzer.load_violent_actions();
        analyzer.load_threat_patterns();
        analyzer
    }

    fn load_violence_patterns(&mut self) {
        let patterns = vec![
            // Direct violence threats
            (r"\bi\s+(will|am\s+going\s+to|plan\s+to)\s+(kill|murder|hurt|harm|beat|attack)", 0.9),
            (r"\bi\s+will\s+(destroy|crush|annihilate|eliminate)\s+you", 0.8),
            (r"\byou\s+(will|are\s+going\s+to|gonna)\s+(die|get\s+hurt|pay)", 0.8),
            
            // Weapon threats
            (r"\bi\s+have\s+a\s+(gun|knife|weapon|bomb)", 0.9),
            (r"\bi\s+will\s+(shoot|stab|bomb|blow\s+up)", 0.9),
            (r"\bput\s+a\s+(bullet|knife)\s+(in|through)", 0.9),
            
            // Physical violence descriptions
            (r"\b(punch|kick|slap|hit)\s+(you|your|them)\s+(in\s+the)", 0.7),
            (r"\bbreak\s+(your|their)\s+(bones|neck|face|legs)", 0.8),
            (r"\bsmash\s+(your|their)\s+(head|face|skull)", 0.8),
            
            // Group violence
            (r"\b(gang|mob|crew)\s+(will|gonna)\s+(get|find|hurt)", 0.7),
            (r"\bwe\s+(will|are\s+going\s+to)\s+(kill|destroy|eliminate)", 0.8),
            (r"\b(riot|uprising|revolution)\s+(against|to\s+destroy)", 0.6),
            
            // Torture and extreme violence
            (r"\b(torture|torment|mutilate|dismember)", 0.9),
            (r"\bmake\s+you\s+(suffer|bleed|scream)", 0.8),
            (r"\bcut\s+(off|out)\s+your", 0.9),
            
            // War and conflict language
            (r"\b(war|battle|fight)\s+(against|with)\s+\w+", 0.5),
            (r"\btime\s+to\s+(fight|battle|war)", 0.6),
            (r"\b(crusade|jihad|holy\s+war)", 0.7),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.violence_patterns.push((regex, weight));
            }
        }
    }

    fn load_weapon_keywords(&mut self) {
        // Firearms
        let firearms = vec![
            ("gun", 0.6), ("rifle", 0.7), ("pistol", 0.7), ("shotgun", 0.7),
            ("firearm", 0.6), ("weapon", 0.5), ("bullet", 0.6), ("ammunition", 0.5),
            ("trigger", 0.4), ("barrel", 0.3), ("scope", 0.3), ("magazine", 0.3),
        ];
        
        // Bladed weapons
        let bladed = vec![
            ("knife", 0.6), ("blade", 0.5), ("sword", 0.5), ("dagger", 0.6),
            ("razor", 0.5), ("machete", 0.7), ("axe", 0.5), ("hatchet", 0.6),
        ];
        
        // Explosives
        let explosives = vec![
            ("bomb", 0.8), ("explosive", 0.7), ("grenade", 0.8), ("dynamite", 0.8),
            ("c4", 0.9), ("tnt", 0.7), ("blast", 0.4), ("explosion", 0.5),
        ];
        
        // Other weapons
        let other_weapons = vec![
            ("bat", 0.4), ("club", 0.4), ("hammer", 0.3), ("crowbar", 0.5),
            ("pipe", 0.3), ("chain", 0.4), ("whip", 0.5), ("taser", 0.6),
        ];

        for (word, weight) in firearms.into_iter()
            .chain(bladed)
            .chain(explosives)
            .chain(other_weapons) {
            self.weapon_keywords.insert(word.to_string(), weight);
        }
    }

    fn load_violent_actions(&mut self) {
        // Physical violence
        let physical = vec![
            ("kill", 0.8), ("murder", 0.9), ("assassinate", 0.9), ("execute", 0.7),
            ("beat", 0.7), ("punch", 0.6), ("kick", 0.6), ("slap", 0.5),
            ("hit", 0.5), ("strike", 0.5), ("smash", 0.6), ("crush", 0.6),
            ("break", 0.4), ("fracture", 0.5), ("injure", 0.5), ("wound", 0.6),
        ];
        
        // Extreme violence
        let extreme = vec![
            ("torture", 0.9), ("mutilate", 0.9), ("dismember", 0.9), ("decapitate", 0.9),
            ("strangle", 0.8), ("choke", 0.7), ("suffocate", 0.8), ("drown", 0.7),
            ("burn", 0.6), ("electrocute", 0.8), ("poison", 0.7), ("overdose", 0.6),
        ];
        
        // Weapons actions
        let weapon_actions = vec![
            ("shoot", 0.7), ("stab", 0.8), ("slash", 0.7), ("cut", 0.4),
            ("slice", 0.5), ("pierce", 0.6), ("impale", 0.8), ("bomb", 0.8),
            ("explode", 0.6), ("detonate", 0.7), ("blow", 0.4),
        ];
        
        // Aggressive actions
        let aggressive = vec![
            ("attack", 0.6), ("assault", 0.7), ("ambush", 0.6), ("raid", 0.5),
            ("invade", 0.5), ("storm", 0.4), ("charge", 0.3), ("rush", 0.2),
            ("fight", 0.4), ("battle", 0.4), ("combat", 0.4), ("war", 0.4),
        ];

        for (word, weight) in physical.into_iter()
            .chain(extreme)
            .chain(weapon_actions)
            .chain(aggressive) {
            self.violent_actions.insert(word.to_string(), weight);
        }
    }

    fn load_threat_patterns(&mut self) {
        let patterns = vec![
            // Direct threats
            (r"\bi\s+will\s+(hurt|harm|kill|destroy)\s+you", 0.9),
            (r"\byou\s+(better|should)\s+(watch\s+out|be\s+careful)", 0.6),
            (r"\byou\s+are\s+(dead|finished|done)", 0.7),
            
            // Conditional threats
            (r"\bif\s+you\s+don't\s+\w+,\s+i\s+will", 0.7),
            (r"\bunless\s+you\s+\w+,\s+(you|i)\s+will", 0.6),
            (r"\bdo\s+\w+\s+or\s+(else|i\s+will)", 0.6),
            
            // Intimidation
            (r"\bi\s+know\s+where\s+you\s+(live|work|go)", 0.8),
            (r"\byou\s+can't\s+(hide|run|escape)", 0.6),
            (r"\bi\s+will\s+find\s+you", 0.7),
            
            // Group threats
            (r"\bwe\s+will\s+(get|find|hurt)\s+you", 0.7),
            (r"\bmy\s+(friends|crew|gang)\s+will", 0.6),
            (r"\byou\s+don't\s+know\s+who\s+you're\s+messing\s+with", 0.6),
        ];

        for (pattern, weight) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.threat_patterns.push((regex, weight));
            }
        }
    }

    pub async fn analyze(&self, content: &str) -> Result<AnalysisResult> {
        let normalized_content = normalize_text(content, self.config.case_sensitive);
        let words: Vec<&str> = normalized_content.split_whitespace().collect();
        
        let mut total_score = 0.0;
        let mut detected_patterns = Vec::new();
        let mut matches = 0;
        let total_patterns = self.violence_patterns.len() + 
                           self.weapon_keywords.len() + 
                           self.violent_actions.len() + 
                           self.threat_patterns.len();

        // Check for violence patterns
        for (pattern, weight) in &self.violence_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("violence_pattern: {}", pattern.as_str()));
            }
        }

        // Check for threat patterns
        for (pattern, weight) in &self.threat_patterns {
            if pattern.is_match(&normalized_content) {
                total_score += weight;
                matches += 1;
                detected_patterns.push(format!("threat_pattern: {}", pattern.as_str()));
            }
        }

        // Analyze weapon references with context
        let weapon_score = self.analyze_weapon_context(&words, &mut detected_patterns);
        if weapon_score > 0.0 {
            total_score += weapon_score;
            matches += 1;
        }

        // Analyze violent actions with intent
        let action_score = self.analyze_violent_actions(&words, &mut detected_patterns);
        if action_score > 0.0 {
            total_score += action_score;
            matches += 1;
        }

        // Check for escalation and severity
        let escalation_score = self.analyze_escalation(&normalized_content, &mut detected_patterns);
        if escalation_score > 0.0 {
            total_score += escalation_score;
            matches += 1;
        }

        // Analyze context for legitimate vs threatening usage
        let context_multiplier = self.analyze_context(&words);
        total_score *= context_multiplier;

        // Normalize final score
        let word_count = words.len() as f64;
        let normalized_score = if word_count > 0.0 {
            (total_score / (word_count / 6.0).max(1.0)).min(1.0)
        } else {
            0.0
        };

        // Calculate confidence
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

    fn analyze_weapon_context(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut weapon_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_weight) = self.weapon_keywords.get(*word) {
                let mut adjusted_weight = base_weight;
                
                // Check for threatening context
                let context_start = if i >= 3 { i - 3 } else { 0 };
                let context_end = std::cmp::min(i + 4, words.len());
                
                let mut has_threat_context = false;
                for j in context_start..context_end {
                    if j != i {
                        let threat_indicators = ["will", "going", "have", "get", "use", "with"];
                        if threat_indicators.contains(&words[j]) {
                            has_threat_context = true;
                            adjusted_weight *= 1.5;
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
                
                weapon_score += adjusted_weight.min(1.0);
                
                if has_threat_context && is_first_person {
                    detected_patterns.push(format!("weapon_threat: {}", word));
                } else if has_threat_context || is_first_person {
                    detected_patterns.push(format!("weapon_reference: {}", word));
                } else {
                    detected_patterns.push(format!("weapon_mention: {}", word));
                }
            }
        }
        
        weapon_score.min(0.8)
    }

    fn analyze_violent_actions(&self, words: &[&str], detected_patterns: &mut Vec<String>) -> f64 {
        let mut action_score = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if let Some(&base_weight) = self.violent_actions.get(*word) {
                let mut adjusted_weight = base_weight;
                
                // Check for targeting (you, them, etc.)
                let context_start = if i >= 2 { i - 2 } else { 0 };
                let context_end = std::cmp::min(i + 3, words.len());
                
                let mut is_targeted = false;
                for j in context_start..context_end {
                    if j != i {
                        let targets = ["you", "your", "them", "their", "him", "her"];
                        if targets.contains(&words[j]) {
                            is_targeted = true;
                            adjusted_weight *= 1.4;
                            break;
                        }
                    }
                }
                
                // Check for intent indicators
                let mut has_intent = false;
                for j in context_start..context_end {
                    if j != i {
                        let intent_words = ["will", "going", "want", "plan", "need"];
                        if intent_words.contains(&words[j]) {
                            has_intent = true;
                            adjusted_weight *= 1.2;
                            break;
                        }
                    }
                }
                
                action_score += adjusted_weight.min(1.0);
                
                if is_targeted && has_intent {
                    detected_patterns.push(format!("targeted_violence: {}", word));
                } else if is_targeted {
                    detected_patterns.push(format!("violent_action_targeted: {}", word));
                } else {
                    detected_patterns.push(format!("violent_action: {}", word));
                }
            }
        }
        
        action_score.min(0.9)
    }

    fn analyze_escalation(&self, content: &str, detected_patterns: &mut Vec<String>) -> f64 {
        let mut escalation_score: f64 = 0.0;
        
        // Check for escalation patterns
        let escalation_patterns = vec![
            (r"\bthis\s+is\s+your\s+last\s+(warning|chance)", 0.7),
            (r"\bnext\s+time\s+i\s+see\s+you", 0.6),
            (r"\bi\s+warned\s+you", 0.5),
            (r"\bnow\s+you've\s+done\s+it", 0.6),
            (r"\bthat's\s+it,\s+you're", 0.6),
            (r"\bi've\s+had\s+enough", 0.5),
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

    fn analyze_context(&self, words: &[&str]) -> f64 {
        let mut context_multiplier: f64 = 1.0;
        
        // Check for gaming/entertainment context
        let gaming_context = ["game", "gaming", "video", "movie", "show", "character", "fiction"];
        let gaming_count = words.iter()
            .filter(|word| gaming_context.contains(word))
            .count();
            
        if gaming_count > 0 {
            context_multiplier *= 0.6; // Reduce score for gaming context
        }
        
        // Check for news/reporting context
        let news_context = ["news", "report", "article", "story", "happened", "occurred"];
        let news_count = words.iter()
            .filter(|word| news_context.contains(word))
            .count();
            
        if news_count > 0 {
            context_multiplier *= 0.7; // Reduce score for news context
        }
        
        // Check for historical context
        let historical_context = ["history", "historical", "past", "ago", "ancient", "medieval"];
        let historical_count = words.iter()
            .filter(|word| historical_context.contains(word))
            .count();
            
        if historical_count > 0 {
            context_multiplier *= 0.8; // Reduce score for historical context
        }
        
        // Check for self-defense context
        let defense_context = ["defense", "protect", "defending", "safety", "security"];
        let defense_count = words.iter()
            .filter(|word| defense_context.contains(word))
            .count();
            
        if defense_count > 0 {
            context_multiplier *= 0.9; // Slightly reduce for defensive context
        }
        
        context_multiplier.max(0.3).min(1.0)
    }

    fn calculate_context_score(&self, words: &[&str]) -> f64 {
        let mut context_score: f64 = 0.6;
        
        // Check for educational context
        let educational_indicators = ["learn", "study", "research", "education", "academic"];
        let educational_count = words.iter()
            .filter(|word| educational_indicators.contains(word))
            .count();
            
        if educational_count > 0 {
            context_score *= 0.8;
        }
        
        // Check for hypothetical context
        let hypothetical_indicators = ["if", "would", "could", "might", "imagine", "suppose"];
        let hypothetical_count = words.iter()
            .filter(|word| hypothetical_indicators.contains(word))
            .count();
            
        if hypothetical_count > 0 {
            context_score *= 0.9;
        }
        
        // Check for past tense (historical/reporting)
        let past_tense = ["was", "were", "had", "did", "happened", "occurred"];
        let past_count = words.iter()
            .filter(|word| past_tense.contains(word))
            .count();
            
        if past_count > 1 {
            context_score *= 0.8;
        }
        
        context_score.max(0.2).min(1.0)
    }
}