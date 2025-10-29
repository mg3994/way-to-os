/// Utility functions for the content moderation system

/// Normalize text for analysis (lowercase, trim, etc.)
pub fn normalize_text(text: &str, case_sensitive: bool) -> String {
    let normalized = text.trim();
    if case_sensitive {
        normalized.to_string()
    } else {
        normalized.to_lowercase()
    }
}

/// Calculate confidence score based on pattern matches and context
pub fn calculate_confidence(matches: usize, total_patterns: usize, context_score: f64) -> f64 {
    if total_patterns == 0 {
        return 0.0;
    }
    
    let match_ratio = matches as f64 / total_patterns as f64;
    let base_confidence = match_ratio * 0.7; // Base confidence from pattern matches
    let context_confidence = context_score * 0.3; // Additional confidence from context
    
    (base_confidence + context_confidence).min(1.0).max(0.0)
}

/// Validate file extension against supported formats
#[allow(dead_code)]
pub fn is_supported_format(filename: &str, supported_formats: &[String]) -> bool {
    if let Some(extension) = filename.split('.').last() {
        supported_formats.iter().any(|format| format.eq_ignore_ascii_case(extension))
    } else {
        false
    }
}

/// Calculate weighted score from multiple components
#[allow(dead_code)]
pub fn calculate_weighted_score(components: &[(f64, f64)]) -> f64 {
    let total_weight: f64 = components.iter().map(|(_, weight)| weight).sum();
    if total_weight == 0.0 {
        return 0.0;
    }
    
    let weighted_sum: f64 = components.iter().map(|(score, weight)| score * weight).sum();
    (weighted_sum / total_weight).min(1.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("  Hello World  ", false), "hello world");
        assert_eq!(normalize_text("  Hello World  ", true), "Hello World");
    }

    #[test]
    fn test_calculate_confidence() {
        assert_eq!(calculate_confidence(0, 0, 0.0), 0.0);
        assert_eq!(calculate_confidence(5, 10, 0.5), 0.5);
        assert!(calculate_confidence(10, 10, 1.0) <= 1.0);
    }

    #[test]
    fn test_is_supported_format() {
        let formats = vec!["jpg".to_string(), "png".to_string()];
        assert!(is_supported_format("image.jpg", &formats));
        assert!(is_supported_format("image.PNG", &formats));
        assert!(!is_supported_format("image.gif", &formats));
        assert!(!is_supported_format("image", &formats));
    }

    #[test]
    fn test_calculate_weighted_score() {
        let components = vec![(0.8, 0.5), (0.6, 0.3), (0.4, 0.2)];
        let score = calculate_weighted_score(&components);
        assert!(score >= 0.0 && score <= 1.0);
    }
}