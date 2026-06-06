//! Character sheet — the loaded persona that encounters are run against

use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::ability::AbilityType;
use crate::log::EncounterLog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSheet {
    pub name: String,
    pub class: CharacterClass,
    pub level: u32,
    pub xp: u64,
    pub trust: f64,
    pub stats: Vec<Stat>,
    pub abilities: Vec<Ability>,
    #[serde(skip)]
    pub log: EncounterLog,
}

impl CharacterSheet {
    pub fn new(name: impl Into<String>, class: CharacterClass) -> Self {
        Self {
            name: name.into(),
            class,
            level: 1,
            xp: 0,
            trust: 50.0,
            stats: vec![
                Stat::new("perception", 50),
                Stat::new("charisma", 50),
                Stat::new("intelligence", 50),
                Stat::new("dexterity", 50),
            ],
            abilities: Vec::new(),
            log: EncounterLog::new(),
        }
    }

    /// Get a stat by name
    pub fn stat(&self, name: &str) -> Option<&Stat> {
        self.stats.iter().find(|s| s.name == name)
    }

    pub fn stat_mut(&mut self, name: &str) -> Option<&mut Stat> {
        self.stats.iter_mut().find(|s| s.name == name)
    }

    /// Get perception value
    pub fn perception(&self) -> u32 {
        self.stat("perception").map(|s| s.value).unwrap_or(50)
    }

    /// Add XP and check for level up (every 500 XP)
    pub fn add_xp(&mut self, amount: u64) -> bool {
        let old_level = self.level;
        self.xp += amount;
        self.level = 1 + (self.xp / 500) as u32;
        self.level > old_level
    }

    /// Update trust score (clamped 0-100)
    pub fn update_trust(&mut self, delta: f64) {
        self.trust = (self.trust + delta).clamp(0.0, 100.0);
    }

    /// Find an ability by name
    pub fn ability(&self, name: &str) -> Option<&Ability> {
        self.abilities.iter().find(|a| a.name == name)
    }

    pub fn ability_mut(&mut self, name: &str) -> Option<&mut Ability> {
        self.abilities.iter_mut().find(|a| a.name == name)
    }

    /// Get abilities of a specific type
    pub fn abilities_of_type(&self, ability_type: AbilityType) -> Vec<&Ability> {
        self.abilities.iter().filter(|a| a.ability_type == ability_type).collect()
    }

    /// Add a new ability
    pub fn add_ability(&mut self, ability: Ability) {
        self.abilities.push(ability);
    }

    /// Total level across all abilities
    pub fn total_ability_levels(&self) -> u32 {
        self.abilities.iter().map(|a| a.level).sum()
    }
}
