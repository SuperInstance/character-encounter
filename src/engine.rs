//! Encounter engine — the main loop

use crate::character::CharacterSheet;
use crate::encounter::Encounter;
use crate::perception::PerceptionCheck;
use crate::ability::AbilityResolution;
use crate::difficulty::DifficultyAssessment;
use crate::log::LogEntry;
use crate::types::EncounterResult;
use crate::types::Difficulty;

pub struct EncounterEngine;

impl EncounterEngine {
    /// Run a single encounter against a character.
    /// `forced_roll` is for testing — pass None for production use.
    pub fn run_encounter(
        character: &mut CharacterSheet,
        input_text: &str,
        forced_roll: Option<u32>,
    ) -> EncounterResult {
        // 1. Perception check → extract intent
        let (intent, _quality) = PerceptionCheck::extract_intent(character, input_text, forced_roll);

        // 2. Build encounter
        let mut encounter = Encounter::new(input_text).with_intent(&intent);

        // 3. Difficulty assessment
        let difficulty = DifficultyAssessment::assess(character, &intent);
        encounter = encounter.with_difficulty(difficulty);

        // 4. Ability resolution
        let ability_match = AbilityResolution::resolve(character, &intent)
            .expect("resolve always returns at least model fallback");

        // 5. Roll for success (trust-based probability)
        let roll = forced_roll.unwrap_or_else(crate::types::roll_d100);
        let success = crate::types::roll_with(character.trust as u32, roll);

        // 6. Calculate rewards/penalties
        let base_xp = 10.0;
        let xp_gained = if success {
            (base_xp * difficulty.xp_multiplier()) as u64
        } else {
            (base_xp * 0.2) as u64 // Small XP even on failure
        };

        let trust_change = if success {
            difficulty.trust_reward()
        } else {
            difficulty.trust_penalty()
        };

        // 7. Apply results
        character.update_trust(trust_change);

        // XP to the character
        let leveled = character.add_xp(xp_gained);

        // XP to the matched ability (if it exists)
        if let Some(ability) = character.ability_mut(&ability_match.ability_name) {
            ability.add_xp(xp_gained);
        }

        // XP to perception stat on every encounter
        if let Some(percep) = character.stat_mut("perception") {
            percep.add_xp(xp_gained / 2);
        }

        // 8. Log the encounter
        let log_entry = LogEntry {
            id: encounter.id.clone(),
            input_text: input_text.to_string(),
            intent: intent.clone(),
            ability_used: ability_match.ability_name.clone(),
            ability_type: ability_match.ability_type,
            success,
            xp_gained,
            trust_change,
            difficulty,
            timestamp: encounter.timestamp,
        };
        character.log.log(log_entry);

        let message = if success {
            format!("✅ Success! Used {} ({}) — +{}xp", ability_match.ability_name, ability_match.ability_type, xp_gained)
        } else {
            format!("❌ Failed with {} ({}) — +{}xp (partial)", ability_match.ability_name, ability_match.ability_type, xp_gained)
        };

        EncounterResult {
            success,
            ability_used: ability_match.ability_name,
            ability_type: ability_match.ability_type,
            xp_gained,
            trust_change,
            difficulty,
            message: if leveled {
                format!("{} 🎉 LEVEL UP! Now level {}", message, character.level)
            } else {
                message
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Ability, CharacterClass};

    fn make_character() -> CharacterSheet {
        let mut char = CharacterSheet::new("Alice", CharacterClass::Assistant);
        char.trust = 80.0;
        char.add_ability(Ability::hardcoded("greeting", "Handle greetings", r"hello|hi|hey"));
        char.add_ability(Ability::hardcoded("translate", "Translate text", r"translate"));
        char.add_ability(Ability::hardcoded("calculate", "Do math", r"calculate|math|compute"));
        char
    }

    #[test]
    fn test_basic_encounter_success() {
        let mut char = make_character();
        // Roll 50 <= trust 80 → success
        let result = EncounterEngine::run_encounter(&mut char, "hello there", Some(50));
        assert!(result.success);
        assert_eq!(result.ability_used, "greeting");
        assert_eq!(result.difficulty, Difficulty::Easy);
        assert!(result.xp_gained > 0);
    }

    #[test]
    fn test_basic_encounter_failure() {
        let mut char = make_character();
        // Roll 90 > trust 80 → failure
        let result = EncounterEngine::run_encounter(&mut char, "hello", Some(90));
        assert!(!result.success);
    }

    #[test]
    fn test_trust_increases_on_success() {
        let mut char = make_character();
        let trust_before = char.trust;
        EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        assert!(char.trust > trust_before);
    }

    #[test]
    fn test_trust_decreases_on_failure() {
        let mut char = make_character();
        let trust_before = char.trust;
        EncounterEngine::run_encounter(&mut char, "hello", Some(90));
        assert!(char.trust < trust_before);
    }

    #[test]
    fn test_xp_gain() {
        let mut char = make_character();
        let xp_before = char.xp;
        EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        // XP is tracked via level-ups; check that log recorded XP
        assert!(char.log.entries()[0].xp_gained > 0);
    }

    #[test]
    fn test_encounter_logged() {
        let mut char = make_character();
        EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        assert_eq!(char.log.len(), 1);
        assert_eq!(char.log.entries()[0].input_text, "hello");
    }

    #[test]
    fn test_multi_encounter_session() {
        let mut char = make_character();
        EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        EncounterEngine::run_encounter(&mut char, "translate this", Some(40));
        EncounterEngine::run_encounter(&mut char, "calculate 2+2", Some(60));
        assert_eq!(char.log.len(), 3);
    }

    #[test]
    fn test_character_level_up() {
        let mut char = make_character();
        char.level = 1;
        char.xp = 0;
        // Each easy encounter gives ~10xp, need 500 per level
        // Run enough to guarantee level up
        for _ in 0..120 {
            EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        }
        assert!(char.level > 1, "expected level > 1, got {}", char.level);
    }

    #[test]
    fn test_stat_growth_from_abilities() {
        let mut char = make_character();
        let percep_before = char.perception();
        // Run many encounters to grow perception stat
        for _ in 0..50 {
            EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        }
        let percep_after = char.perception();
        assert!(percep_after >= percep_before);
    }

    #[test]
    fn test_ability_level_up() {
        let mut char = make_character();
        for _ in 0..30 {
            EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        }
        let greeting = char.ability("greeting").unwrap();
        assert!(greeting.level > 1 || greeting.xp > 0);
    }

    #[test]
    fn test_novel_encounter_forces_model() {
        let char = CharacterSheet::new("Empty", CharacterClass::Assistant);
        let difficulty = DifficultyAssessment::assess(&char, "something completely novel");
        assert_eq!(difficulty, Difficulty::Novel);
    }

    #[test]
    fn test_biography_generation() {
        let mut char = make_character();
        EncounterEngine::run_encounter(&mut char, "hello", Some(50));
        EncounterEngine::run_encounter(&mut char, "translate", Some(90)); // fail
        let bio = char.log.generate_biography();
        assert!(bio.contains("Encounter Biography"));
        assert!(bio.contains("2 encounters"));
    }

    #[test]
    fn test_difficulty_novel_high_xp() {
        let mut char = CharacterSheet::new("Empty", CharacterClass::Assistant);
        char.trust = 50.0;
        let result = EncounterEngine::run_encounter(&mut char, "novel thing", Some(25));
        assert_eq!(result.difficulty, Difficulty::Novel);
        // Novel success should give 5x XP
        let expected_xp = (10.0 * 5.0) as u64;
        assert_eq!(result.xp_gained, expected_xp);
    }
}
