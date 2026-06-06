//! Perception check — intent extraction as a stat-based ability check

use crate::character::CharacterSheet;

/// Quality of intent extraction based on perception roll
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentQuality {
    Perfect,   // High perception roll — accurate 3-8 word intent
    Adequate,  // Moderate roll — mostly accurate, may miss nuance
    Noisy,     // Low roll — misunderstood or vague intent
}

pub struct PerceptionCheck;

impl PerceptionCheck {
    /// Extract intent from input text using perception stat.
    /// Returns (intent_string, quality).
    pub fn extract_intent(character: &CharacterSheet, input_text: &str, forced_roll: Option<u32>) -> (String, IntentQuality) {
        let perception = character.perception();
        let roll = forced_roll.unwrap_or_else(crate::types::roll_d100);
        let success = roll <= perception;

        let quality = if perception >= 80 && success {
            IntentQuality::Perfect
        } else if perception >= 50 || success {
            IntentQuality::Adequate
        } else {
            IntentQuality::Noisy
        };

        let intent = match quality {
            IntentQuality::Perfect => Self::compress_intent(input_text),
            IntentQuality::Adequate => Self::compress_intent(input_text), // same but caller knows quality
            IntentQuality::Noisy => Self::noisy_intent(input_text),
        };

        (intent, quality)
    }

    /// Compress input to 3-8 word intent
    fn compress_intent(text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return "unknown intent".to_string();
        }
        let n = words.len().min(8).max(1);
        words[..n].join(" ")
    }

    /// Add noise to intent for low perception
    fn noisy_intent(text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return "unclear intent".to_string();
        }
        if words.len() <= 3 {
            return format!("unclear: {}", text);
        }
        // Drop some words and add uncertainty
        let kept: Vec<&str> = words.iter().step_by(2).copied().take(4).collect();
        if kept.is_empty() {
            return format!("unclear: {}", words[0]);
        }
        kept.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CharacterClass;

    #[test]
    fn test_perfect_perception() {
        let mut char = CharacterSheet::new("Test", CharacterClass::Assistant);
        char.stat_mut("perception").unwrap().value = 90;
        let (intent, quality) = PerceptionCheck::extract_intent(&char, "translate this text to Japanese please", Some(85));
        assert_eq!(quality, IntentQuality::Perfect);
        assert!(!intent.is_empty());
    }

    #[test]
    fn test_noisy_perception() {
        let mut char = CharacterSheet::new("Test", CharacterClass::Assistant);
        char.stat_mut("perception").unwrap().value = 10;
        let (_, quality) = PerceptionCheck::extract_intent(&char, "help me write a Rust program", Some(50));
        assert_eq!(quality, IntentQuality::Noisy);
    }

    #[test]
    fn test_adequate_perception() {
        let char = CharacterSheet::new("Test", CharacterClass::Assistant);
        let (_, quality) = PerceptionCheck::extract_intent(&char, "some request", Some(50));
        assert_eq!(quality, IntentQuality::Adequate);
    }
}
