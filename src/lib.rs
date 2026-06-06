//! character-encounter: RPG encounter engine for character sheets (.nail bundles)
//!
//! When a character sheet is loaded, this is the engine that runs encounters.
//! Each encounter is a user request that the character must handle using their abilities.

pub mod encounter;
pub mod engine;
pub mod perception;
pub mod ability;
pub mod log;
pub mod difficulty;
pub mod character;
pub mod types;

pub use encounter::Encounter;
pub use engine::EncounterEngine;
pub use perception::PerceptionCheck;
pub use ability::{AbilityResolution, AbilityMatch, AbilityType};
pub use log::EncounterLog;
pub use difficulty::DifficultyAssessment;
pub use character::CharacterSheet;
pub use types::*;
