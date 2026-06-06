# character-encounter

RPG encounter engine that transforms agent interactions into stat-based ability checks with trust, XP, and leveling.

## Why This Exists

Most agent frameworks treat every request identically — same latency path, same confidence, same cost. That's wrong. An agent that's handled a thousand greetings shouldn't process them the same way it handles a novel request it's never seen. This crate borrows from RPG game design: every interaction is an *encounter* resolved through ability checks, perception rolls, and trust-weighted probability. The result is a character that visibly grows — leveling up abilities it uses often, building trust through success, and degrading through failure.

The key insight: **ability resolution is a tiered fallback system**. Hardcoded regex matches fire in microseconds. Learned embedding matches fire in under a millisecond. Only truly novel requests fall through to expensive LLM calls. This isn't optimization — it's the architecture.

## Architecture

```text
Input Text
    │
    ▼
PerceptionCheck ─── extract intent (stat-based roll)
    │
    ▼
DifficultyAssessment ─── how many abilities match?
    │
    ▼
AbilityResolution ─── hardcoded → learned → hybrid → model
    │                       (0ms)   (<1ms)   (1ms)   (500ms)
    ▼
Trust-weighted Roll ─── success/failure probability
    │
    ▼
XP & Trust Updates ─── character grows
    │
    ▼
EncounterLog ─── full history, biography generation
```

### Key Types

- **`Encounter`** — A single user request with context, timestamp, and extracted intent
- **`EncounterEngine`** — The main loop: perception → difficulty → ability → roll → rewards
- **`PerceptionCheck`** — Extracts intent from raw text; quality depends on perception stat
- **`AbilityResolution`** — Four-tier ability matcher (hardcoded → learned → hybrid → model)
- **`DifficultyAssessment`** — Auto-scales difficulty based on how well abilities cover the intent
- **`CharacterSheet`** — The persona: stats, abilities, trust, level, XP, encounter log
- **`EncounterLog`** — Full encounter history with filtering and biography generation

### Ability Resolution Tiers

| Tier | Type | Latency | When It Fires |
|------|------|---------|---------------|
| 1 | `Hardcoded` | ~0ms | Regex pattern match |
| 2 | `Learned` | <1ms | Embedding cosine similarity ≥ threshold |
| 3 | `Hybrid` | ~1ms | Regex OR embedding match |
| 4 | `Model` | ~500ms | Fallback for novel requests |

### Difficulty & Rewards

| Difficulty | XP Multiplier | Trust Reward | Trust Penalty | When |
|-----------|---------------|--------------|---------------|------|
| Easy | 1.0× | +0.5 | −5.0 | Hardcoded match |
| Medium | 1.5× | +1.5 | −3.0 | Low-confidence learned match |
| Hard | 2.5× | +3.0 | −1.5 | Hybrid match |
| Novel | 5.0× | +5.0 | −0.5 | Model fallback only |

Novel encounters penalize trust the least — you shouldn't be punished for not knowing something. But they reward the most when you succeed.

## Usage

```rust
use character_encounter::{EncounterEngine, CharacterSheet, Difficulty};
use character_encounter::types::{Ability, CharacterClass};

// Create a character
let mut char = CharacterSheet::new("Alice", CharacterClass::Assistant);
char.trust = 80.0;

// Teach it abilities
char.add_ability(Ability::hardcoded("greeting", "Handle greetings", r"hello|hi|hey"));
char.add_ability(Ability::hardcoded("translate", "Translate text", r"translate"));

// Run encounters — forced_roll is for testing, pass None in production
let result = EncounterEngine::run_encounter(&mut char, "hello there", Some(50));
assert!(result.success);
assert_eq!(result.ability_used, "greeting");
assert_eq!(result.difficulty, Difficulty::Easy);

// Failed encounter (roll 90 > trust 80)
let result = EncounterEngine::run_encounter(&mut char, "hello", Some(90));
assert!(!result.success);

// Novel request falls through to model fallback
let empty = CharacterSheet::new("Empty", CharacterClass::Assistant);
let result = EncounterEngine::run_encounter(&mut empty, "something weird", Some(25));
assert_eq!(result.difficulty, Difficulty::Novel);

// Generate a biography from encounter history
let bio = char.log.generate_biography();
println!("{}", bio);
```

## API Reference

### `Encounter`
- `Encounter::new(input_text)` — Create from raw input
- `.with_context(ctx)` — Attach conversation history + environment state
- `.with_intent(intent)` — Set extracted intent
- `.with_difficulty(diff)` — Set assessed difficulty

### `EncounterEngine`
- `EncounterEngine::run_encounter(character, input_text, forced_roll)` — Run full encounter pipeline. Returns `EncounterResult`.

### `PerceptionCheck`
- `PerceptionCheck::extract_intent(character, input, forced_roll)` — Returns `(intent_string, IntentQuality)`. Quality: `Perfect` / `Adequate` / `Noisy`.

### `AbilityResolution`
- `AbilityResolution::resolve(character, intent)` — Returns `Option<AbilityMatch>`. Always returns at least model fallback.
- `AbilityResolution::simple_embedding(text)` — Bag-of-words hash embedding (64-dim)
- `AbilityResolution::cosine_similarity(a, b)` — Cosine similarity between vectors

### `DifficultyAssessment`
- `DifficultyAssessment::assess(character, intent)` → `Difficulty`
- `DifficultyAssessment::count_matches(character, intent)` → `usize`

### `CharacterSheet`
- `CharacterSheet::new(name, class)` — Fresh level-1 character
- `.add_ability(ability)`, `.ability(name)`, `.ability_mut(name)`
- `.add_xp(amount)` → bool (leveled up?)
- `.update_trust(delta)` — Clamped to [0, 100]
- `.perception()` — Get perception stat value
- `.total_ability_levels()` — Sum of all ability levels

### `EncounterLog`
- `.log(entry)`, `.entries()`, `.len()`
- `.successes()`, `.failures()`, `.by_ability_type(ty)`, `.by_difficulty(diff)`
- `.total_xp()`, `.success_rate()`
- `.generate_biography()` — Human-readable encounter history

### Types
- `Difficulty` — `Easy | Medium | Hard | Novel` with XP multipliers and trust modifiers
- `Ability` — `.hardcoded(name, desc, pattern)`, `.learned(name, desc, embedding)`
- `AbilityType` — `Hardcoded | Learned | Hybrid | Model`
- `EncounterResult` — success, ability used, XP gained, trust change, difficulty
- `CharacterClass` — `Assistant | Specialist | Generalist | Custom(String)`

## The Deeper Idea

This crate is the runtime engine for the `.nail` character system. A `.nail` bundle defines a character sheet — stats, abilities, trust thresholds — that gets loaded by `character-sheet` and run by this crate. Every encounter leaves a mark: abilities level up, trust shifts, the biography grows. Over time, a character becomes specialized in what it's good at and transparent about what it isn't.

The trust-as-probability model is deliberate: high trust means high success rate, which means fast paths execute reliably. Low trust forces more failures, which slows the character down — a natural backpressure mechanism that mirrors how real trust works.

## Related Crates

- [`character-sheet`](../character-sheet) — The `.nail` bundle format and bidirectional converter
- [`pincher-flux-bridge`](../pincher-flux-bridge) — Bridges reflex actions to flux IR for compilation
- [`position-aware-embed`](../position-aware-embed) — The embedding engine behind learned ability matching
