//! Ability resolution — matching intents to abilities

use regex::Regex;
use crate::character::CharacterSheet;

/// How an ability was matched
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AbilityType {
    Hardcoded,  // Regex match — zero latency, always fires
    Learned,    // Embedding similarity — <1ms with pre-computed vectors
    Hybrid,     // Regex first, embedding fallback
    Model,      // LLM fallback — expensive, slow, handles anything
}

impl std::fmt::Display for AbilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbilityType::Hardcoded => write!(f, "hardcoded"),
            AbilityType::Learned => write!(f, "learned"),
            AbilityType::Hybrid => write!(f, "hybrid"),
            AbilityType::Model => write!(f, "model"),
        }
    }
}

/// Result of ability matching
#[derive(Debug, Clone)]
pub struct AbilityMatch {
    pub ability_name: String,
    pub ability_type: AbilityType,
    pub confidence: f64,
    pub latency_estimate_ms: f64,
}

pub struct AbilityResolution;

impl AbilityResolution {
    /// Try to match intent against character's abilities.
    /// Order: hardcoded (regex) → learned (embedding) → hybrid → model fallback
    pub fn resolve(character: &CharacterSheet, intent: &str) -> Option<AbilityMatch> {
        // 1. Try hardcoded (regex) — zero latency
        if let Some(m) = Self::match_hardcoded(character, intent) {
            return Some(m);
        }

        // 2. Try learned (embedding) — <1ms
        if let Some(m) = Self::match_learned(character, intent, 0.5) {
            return Some(m);
        }

        // 3. Try hybrid (looser regex + lower embedding threshold)
        if let Some(m) = Self::match_hybrid(character, intent) {
            return Some(m);
        }

        // 4. Model fallback — always matches but expensive
        Some(AbilityMatch {
            ability_name: "fallback_reasoning".to_string(),
            ability_type: AbilityType::Model,
            confidence: 0.3,
            latency_estimate_ms: 500.0,
        })
    }

    fn match_hardcoded(character: &CharacterSheet, intent: &str) -> Option<AbilityMatch> {
        let intent_lower = intent.to_lowercase();
        for ability in &character.abilities {
            if ability.ability_type != AbilityType::Hardcoded {
                continue;
            }
            if let Some(pattern) = &ability.pattern {
                if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                    if re.is_match(&intent_lower) {
                        return Some(AbilityMatch {
                            ability_name: ability.name.clone(),
                            ability_type: AbilityType::Hardcoded,
                            confidence: 1.0,
                            latency_estimate_ms: 0.0,
                        });
                    }
                }
                // Also do simple substring match as fallback
                if intent_lower.contains(&pattern.to_lowercase()) {
                    return Some(AbilityMatch {
                        ability_name: ability.name.clone(),
                        ability_type: AbilityType::Hardcoded,
                        confidence: 0.9,
                        latency_estimate_ms: 0.0,
                    });
                }
            }
        }
        None
    }

    fn match_learned(character: &CharacterSheet, intent: &str, threshold: f64) -> Option<AbilityMatch> {
        let intent_vec = Self::simple_embedding(intent);
        let mut best: Option<AbilityMatch> = None;

        for ability in &character.abilities {
            if ability.ability_type != AbilityType::Learned {
                continue;
            }
            if let Some(emb) = &ability.embedding {
                let sim = Self::cosine_similarity(&intent_vec, emb);
                if sim >= threshold {
                    if best.as_ref().map_or(true, |b| sim > b.confidence) {
                        best = Some(AbilityMatch {
                            ability_name: ability.name.clone(),
                            ability_type: AbilityType::Learned,
                            confidence: sim,
                            latency_estimate_ms: 0.5,
                        });
                    }
                }
            }
        }
        best
    }

    fn match_hybrid(character: &CharacterSheet, intent: &str) -> Option<AbilityMatch> {
        // Try hybrid abilities (regex + embedding combined)
        let intent_lower = intent.to_lowercase();
        let intent_vec = Self::simple_embedding(intent);

        for ability in &character.abilities {
            if ability.ability_type != AbilityType::Hybrid {
                continue;
            }

            let regex_match = ability.pattern.as_ref().map(|p| {
                Regex::new(&format!("(?i){}", p))
                    .map(|re| re.is_match(&intent_lower))
                    .unwrap_or_else(|_| intent_lower.contains(&p.to_lowercase()))
            }).unwrap_or(false);

            let emb_match = ability.embedding.as_ref().map(|emb| {
                Self::cosine_similarity(&intent_vec, emb)
            }).unwrap_or(0.0);

            if regex_match || emb_match >= 0.3 {
                let confidence = if regex_match && emb_match >= 0.3 {
                    0.95
                } else if regex_match {
                    0.8
                } else {
                    emb_match
                };

                return Some(AbilityMatch {
                    ability_name: ability.name.clone(),
                    ability_type: AbilityType::Hybrid,
                    confidence,
                    latency_estimate_ms: 1.0,
                });
            }
        }
        None
    }

    /// Simple bag-of-words embedding (placeholder for real embeddings)
    pub fn simple_embedding(text: &str) -> Vec<f32> {
        // Use word hashes as a simple embedding
        let mut vec = vec![0.0f32; 64];
        for word in text.to_lowercase().split_whitespace() {
            let hash = simple_hash(word);
            for i in 0..64 {
                vec[i] += ((hash >> i) & 1) as f32;
            }
        }
        // Normalize
        let mag: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt().max(0.001);
        vec.iter_mut().for_each(|v| *v /= mag);
        vec
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|v| v * v).sum::<f32>().sqrt().max(0.001);
        let mag_b: f32 = b.iter().map(|v| v * v).sum::<f32>().sqrt().max(0.001);
        (dot / (mag_a * mag_b)) as f64
    }
}

/// Simple FNV-1a-like hash for words
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Ability as AbilityDef, CharacterClass};

    fn make_char() -> CharacterSheet {
        let mut char = CharacterSheet::new("Test", CharacterClass::Assistant);
        char.add_ability(AbilityDef::hardcoded("greeting", "Handle greetings", r"hello|hi|hey|greet"));
        char.add_ability(AbilityDef::hardcoded("translate", "Translate text", r"translate"));
        let emb = AbilityResolution::simple_embedding("write code in rust");
        char.add_ability(AbilityDef::learned("code_writer", "Write code", emb));
        char.add_ability(AbilityDef::new("hybrid_search", "Search hybrid", AbilityType::Hybrid));
        char
    }

    #[test]
    fn test_hardcoded_match() {
        let char = make_char();
        let m = AbilityResolution::resolve(&char, "hello there").unwrap();
        assert_eq!(m.ability_name, "greeting");
        assert_eq!(m.ability_type, AbilityType::Hardcoded);
    }

    #[test]
    fn test_learned_match() {
        let char = make_char();
        let m = AbilityResolution::resolve(&char, "write some rust code please").unwrap();
        assert_eq!(m.ability_name, "code_writer");
        assert_eq!(m.ability_type, AbilityType::Learned);
    }

    #[test]
    fn test_model_fallback() {
        let char = CharacterSheet::new("Empty", CharacterClass::Assistant);
        let m = AbilityResolution::resolve(&char, "something totally novel").unwrap();
        assert_eq!(m.ability_type, AbilityType::Model);
    }

    #[test]
    fn test_hybrid_match() {
        let mut char = CharacterSheet::new("Test", CharacterClass::Assistant);
        let mut a = AbilityDef::new("hybrid_translate", "Hybrid translate", AbilityType::Hybrid);
        a.pattern = Some("translate|convert language".to_string());
        let emb = AbilityResolution::simple_embedding("translate text from english");
        a.embedding = Some(emb);
        char.add_ability(a);

        let m = AbilityResolution::resolve(&char, "translate this document").unwrap();
        assert_eq!(m.ability_name, "hybrid_translate");
        assert_eq!(m.ability_type, AbilityType::Hybrid);
    }

    #[test]
    fn test_cosine_similarity_same() {
        let v = vec![1.0f32, 0.0, 1.0, 0.0];
        assert!((AbilityResolution::cosine_similarity(&v, &v) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0f32, 0.0];
        let b = vec![0.0f32, 1.0];
        assert!(AbilityResolution::cosine_similarity(&a, &b).abs() < 0.001);
    }
}
