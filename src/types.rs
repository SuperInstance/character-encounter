//! Shared types used across the encounter system

use serde::{Deserialize, Serialize};

/// Difficulty level for an encounter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Novel,
}

impl Difficulty {
    /// XP multiplier based on difficulty
    pub fn xp_multiplier(self) -> f64 {
        match self {
            Difficulty::Easy => 1.0,
            Difficulty::Medium => 1.5,
            Difficulty::Hard => 2.5,
            Difficulty::Novel => 5.0,
        }
    }

    /// Base trust change on failure
    pub fn trust_penalty(self) -> f64 {
        match self {
            Difficulty::Easy => -5.0,
            Difficulty::Medium => -3.0,
            Difficulty::Hard => -1.5,
            Difficulty::Novel => -0.5,
        }
    }

    /// Base trust change on success
    pub fn trust_reward(self) -> f64 {
        match self {
            Difficulty::Easy => 0.5,
            Difficulty::Medium => 1.5,
            Difficulty::Hard => 3.0,
            Difficulty::Novel => 5.0,
        }
    }
}

/// Result of an encounter execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterResult {
    pub success: bool,
    pub ability_used: String,
    pub ability_type: crate::ability::AbilityType,
    pub xp_gained: u64,
    pub trust_change: f64,
    pub difficulty: Difficulty,
    pub message: String,
}

/// Character class
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterClass {
    Assistant,
    Specialist,
    Generalist,
    Custom(String),
}

/// A stat value with a name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub value: u32,
    pub xp_invested: u64,
}

impl Stat {
    pub fn new(name: impl Into<String>, value: u32) -> Self {
        Self {
            name: name.into(),
            value,
            xp_invested: 0,
        }
    }

    /// Add XP to this stat; every 100 XP invested increases value by 1
    pub fn add_xp(&mut self, amount: u64) -> bool {
        self.xp_invested += amount;
        let new_value = self.value + (self.xp_invested / 100) as u32;
        let grew = new_value > self.value;
        self.value = new_value;
        self.xp_invested %= 100;
        grew
    }
}

/// An ability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub level: u32,
    pub xp: u64,
    pub uses: u64,
    pub ability_type: crate::ability::AbilityType,
    /// Regex pattern for hardcoded abilities
    pub pattern: Option<String>,
    /// Embedding vector for learned abilities (simplified as f32 slice)
    pub embedding: Option<Vec<f32>>,
    /// Description of what this ability does
    pub description: String,
}

impl Ability {
    pub fn new(name: impl Into<String>, description: impl Into<String>, ability_type: crate::ability::AbilityType) -> Self {
        Self {
            name: name.into(),
            level: 1,
            xp: 0,
            uses: 0,
            ability_type,
            pattern: None,
            embedding: None,
            description: description.into(),
        }
    }

    pub fn hardcoded(name: impl Into<String>, description: impl Into<String>, pattern: impl Into<String>) -> Self {
        let mut a = Self::new(name, description, crate::ability::AbilityType::Hardcoded);
        a.pattern = Some(pattern.into());
        a
    }

    pub fn learned(name: impl Into<String>, description: impl Into<String>, embedding: Vec<f32>) -> Self {
        let mut a = Self::new(name, description, crate::ability::AbilityType::Learned);
        a.embedding = Some(embedding);
        a
    }

    /// Add XP to ability; level up every 200 XP
    pub fn add_xp(&mut self, amount: u64) -> bool {
        self.xp += amount;
        self.uses += 1;
        let new_level = 1 + (self.xp / 200) as u32;
        let leveled = new_level > self.level;
        self.level = new_level;
        leveled
    }
}

/// Simple dice roll: 1d100
pub fn roll_d100() -> u32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    // Simple LCG for deterministic-ish rolls in tests
    (nanos % 100) + 1
}

/// Roll against a stat value (stat out of 100 maps to success probability)
pub fn roll_against(stat_value: u32) -> bool {
    let roll = roll_d100();
    roll <= stat_value
}

/// Deterministic roll for testing
pub fn roll_with(stat_value: u32, forced_roll: u32) -> bool {
    forced_roll <= stat_value
}
