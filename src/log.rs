//! Encounter log — full history of encounters

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::Difficulty;
use crate::ability::AbilityType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub input_text: String,
    pub intent: String,
    pub ability_used: String,
    pub ability_type: AbilityType,
    pub success: bool,
    pub xp_gained: u64,
    pub trust_change: f64,
    pub difficulty: Difficulty,
    pub timestamp: DateTime<Utc>,
}

impl LogEntry {
    /// Generate a biography-style summary line
    pub fn to_biography_line(&self) -> String {
        let outcome = if self.success { "succeeded" } else { "failed" };
        format!(
            "{}: Faced a {:?} encounter ('{}'), {} using {} ({}+{}xp)",
            self.timestamp.format("%Y-%m-%d %H:%M"),
            self.difficulty,
            self.intent,
            outcome,
            self.ability_used,
            if self.trust_change >= 0.0 { "+" } else { "" },
            self.trust_change,
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncounterLog {
    entries: Vec<LogEntry>,
}

impl EncounterLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn log(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Filter by success/failure
    pub fn successes(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.success).collect()
    }

    pub fn failures(&self) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| !e.success).collect()
    }

    /// Filter by ability type
    pub fn by_ability_type(&self, ability_type: AbilityType) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.ability_type == ability_type).collect()
    }

    /// Filter by difficulty
    pub fn by_difficulty(&self, difficulty: Difficulty) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.difficulty == difficulty).collect()
    }

    /// Total XP gained
    pub fn total_xp(&self) -> u64 {
        self.entries.iter().map(|e| e.xp_gained).sum()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        self.successes().len() as f64 / self.entries.len() as f64
    }

    /// Generate a biography from all entries
    pub fn generate_biography(&self) -> String {
        if self.entries.is_empty() {
            return "No encounters recorded yet.".to_string();
        }

        let total = self.entries.len();
        let successes = self.successes().len();
        let failures = self.failures().len();
        let total_xp = self.total_xp();
        let success_rate = self.success_rate() * 100.0;

        let mut bio = format!(
            "=== Encounter Biography ===\n\
             {} encounters: {} succeeded, {} failed ({:.1}% success rate)\n\
             Total XP earned: {}\n\n\
             Notable moments:\n",
            total, successes, failures, success_rate, total_xp,
        );

        for entry in &self.entries {
            bio.push_str(&format!("- {}\n", entry.to_biography_line()));
        }

        bio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(success: bool, difficulty: Difficulty) -> LogEntry {
        LogEntry {
            id: "test".to_string(),
            input_text: "test input".to_string(),
            intent: "test intent".to_string(),
            ability_used: "test_ability".to_string(),
            ability_type: AbilityType::Hardcoded,
            success,
            xp_gained: if success { 10 } else { 0 },
            trust_change: if success { 1.0 } else { -2.0 },
            difficulty,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_log_and_query() {
        let mut log = EncounterLog::new();
        log.log(make_entry(true, Difficulty::Easy));
        log.log(make_entry(false, Difficulty::Hard));
        log.log(make_entry(true, Difficulty::Medium));

        assert_eq!(log.len(), 3);
        assert_eq!(log.successes().len(), 2);
        assert_eq!(log.failures().len(), 1);
        assert_eq!(log.total_xp(), 20);
    }

    #[test]
    fn test_biography_generation() {
        let mut log = EncounterLog::new();
        log.log(make_entry(true, Difficulty::Easy));
        let bio = log.generate_biography();
        assert!(bio.contains("Encounter Biography"));
        assert!(bio.contains("1 encounters"));
    }
}
