//! Encounter — a single user request to be handled by a character

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::Difficulty;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub id: String,
    pub input_text: String,
    pub intent: Option<String>,
    pub difficulty: Option<Difficulty>,
    pub context: EncounterContext,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterContext {
    pub conversation_history: Vec<String>,
    pub environment_state: std::collections::HashMap<String, String>,
}

impl Default for EncounterContext {
    fn default() -> Self {
        Self {
            conversation_history: Vec::new(),
            environment_state: std::collections::HashMap::new(),
        }
    }
}

impl Encounter {
    pub fn new(input_text: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input_text: input_text.into(),
            intent: None,
            difficulty: None,
            context: EncounterContext::default(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_context(mut self, context: EncounterContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = Some(difficulty);
        self
    }
}
