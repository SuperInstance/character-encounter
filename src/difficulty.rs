//! Difficulty assessment — auto-scale encounter difficulty based on ability coverage

use crate::character::CharacterSheet;
use crate::ability::{AbilityResolution, AbilityType};
use crate::types::Difficulty;

pub struct DifficultyAssessment;

impl DifficultyAssessment {
    /// Assess difficulty based on how many abilities match the intent.
    /// Many matches = easy, few = hard, none = novel
    pub fn assess(character: &CharacterSheet, intent: &str) -> Difficulty {
        let match_result = AbilityResolution::resolve(character, intent);

        match match_result {
            Some(m) => match m.ability_type {
                AbilityType::Hardcoded => Difficulty::Easy,
                AbilityType::Learned if m.confidence > 0.8 => Difficulty::Easy,
                AbilityType::Learned => Difficulty::Medium,
                AbilityType::Hybrid if m.confidence > 0.9 => Difficulty::Easy,
                AbilityType::Hybrid => Difficulty::Medium,
                AbilityType::Model => Difficulty::Novel,
            },
            None => Difficulty::Novel,
        }
    }

    /// Count how many abilities could potentially match (for analytics)
    pub fn count_matches(character: &CharacterSheet, intent: &str) -> usize {
        let mut count = 0;
        let intent_lower = intent.to_lowercase();

        for ability in &character.abilities {
            match ability.ability_type {
                AbilityType::Hardcoded => {
                    if let Some(pattern) = &ability.pattern {
                        if intent_lower.contains(&pattern.to_lowercase()) {
                            count += 1;
                        }
                    }
                }
                AbilityType::Learned => {
                    if let Some(emb) = &ability.embedding {
                        let intent_vec = AbilityResolution::simple_embedding(intent);
                        let sim = AbilityResolution::cosine_similarity(&intent_vec, emb);
                        if sim > 0.3 {
                            count += 1;
                        }
                    }
                }
                AbilityType::Hybrid => {
                    count += 1; // Simplified
                }
                AbilityType::Model => {}
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Ability, CharacterClass};

    fn make_char() -> CharacterSheet {
        let mut char = CharacterSheet::new("Test", CharacterClass::Assistant);
        char.add_ability(Ability::hardcoded("greeting", "Handle greetings", r"hello|hi|hey"));
        char.add_ability(Ability::hardcoded("translate", "Translate text", r"translate"));
        let emb = AbilityResolution::simple_embedding("translate hello");
        char.add_ability(Ability::learned("translate_learned", "Also translate", emb));
        char
    }

    #[test]
    fn test_easy_difficulty() {
        let char = make_char();
        assert_eq!(DifficultyAssessment::assess(&char, "hello there"), Difficulty::Easy);
    }

    #[test]
    fn test_novel_difficulty() {
        let char = CharacterSheet::new("Empty", CharacterClass::Assistant);
        assert_eq!(DifficultyAssessment::assess(&char, "something weird"), Difficulty::Novel);
    }

    #[test]
    fn test_count_matches() {
        let char = make_char();
        let count = DifficultyAssessment::count_matches(&char, "translate hello");
        assert!(count >= 2, "expected >= 2 matches, got {}", count);
    }
}
