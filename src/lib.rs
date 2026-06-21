/*MIT License

Copyright (c) 2018 sn99

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

pub mod extra;

use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use rand::Rng;
use serde::{Deserialize, Serialize};

pub const TEAM_PATH: &str = "resources/pokemons.json";

pub fn write_team_to_file<P: AsRef<Path>>(
    path: P,
    team: &PokemonsList,
) -> Result<(), Box<dyn Error>> {
    fs::write(path, serde_json::to_string_pretty(team)?)?;
    Ok(())
}

pub fn read_team_from_file<P: AsRef<Path>>(path: P) -> Result<PokemonsList, Box<dyn Error>> {
    let file = File::open(path)?;
    let team = serde_json::from_reader(file)?;
    Ok(team)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PokemonsList {
    #[serde(rename = "pokemons")]
    pub pokeball: Vec<Pokemon>,
}

impl Default for PokemonsList {
    fn default() -> Self {
        Self {
            pokeball: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pokemon {
    pub name: String,
    #[serde(rename = "moves")]
    pub moves_name: Vec<String>,
    pub health: i64,
    #[serde(rename = "type")]
    pub pokemon_type: i64,
}

/// Result of applying an attack for battle UI / logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackResult {
    pub flavor: String,
    pub damage_dealt: i64,
    pub critical: bool,
    pub blocked: bool,
}

impl Pokemon {
    /// Build a Pokemon without interactive input.
    pub fn with_stats(
        name: impl Into<String>,
        moves_name: Vec<String>,
        health: i64,
        pokemon_type: i64,
    ) -> Self {
        Pokemon {
            name: name.into(),
            moves_name,
            health,
            pokemon_type,
        }
    }

    pub fn edit(&mut self, moves: Vec<String>, health: i64) {
        self.health = health;
        self.moves_name = moves;
    }

    pub fn edit_full(&mut self, name: String, moves: Vec<String>, health: i64, pokemon_type: i64) {
        self.name = name;
        self.moves_name = moves;
        self.health = health;
        self.pokemon_type = pokemon_type;
    }

    /// Apply randomized combat damage. `critical` when chance == 0 (same as original).
    /// Returns flavor text + damage; does not print.
    pub fn take_hit(&mut self, critical_chance: i32) -> AttackResult {
        let mut rng = rand::thread_rng();
        let base = rng.gen_range(80..100);
        let critical = critical_chance == 0;
        let extra = if critical {
            rng.gen_range(10..30)
        } else {
            0
        };
        let damage_dealt = base + extra;
        self.health -= damage_dealt;
        AttackResult {
            flavor: random_message_str(),
            damage_dealt,
            critical,
            blocked: false,
        }
    }

    /// Apply a fixed damage amount (deterministic; useful in tests).
    pub fn apply_damage(&mut self, amount: i64, critical: bool) {
        if critical {
            self.health -= amount + 20;
        } else {
            self.health -= amount;
        }
    }

    /// Returns true when this Pokemon can no longer battle (health <= 0).
    pub fn health_check(&self) -> bool {
        self.is_fainted()
    }

    pub fn is_fainted(&self) -> bool {
        self.health <= 0
    }

    pub fn health_ratio(&self, max_health: i64) -> f64 {
        if max_health <= 0 {
            return 0.0;
        }
        (self.health.max(0) as f64 / max_health as f64).clamp(0.0, 1.0)
    }
}

pub fn random_message_str() -> String {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
        0 => "We can do it!".into(),
        1 => "Never give up!".into(),
        2 => "Be the very best!".into(),
        _ => "Till the end we shall dance!".into(),
    }
}

/// Roll whether an attack is blocked (~1/8, matching original).
pub fn roll_block() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..8) == 0
}

/// Roll critical chance (~1/9, matching original).
pub fn roll_critical_chance() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..9)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extra::{is_valid_choice, parse_i64, parse_moves};
    use std::io::Write;

    fn sample_pikachu() -> Pokemon {
        Pokemon::with_stats(
            "Pikachu",
            vec![
                "IronTail".into(),
                "ElectricBall".into(),
                "ElectricShock".into(),
                "Tackle".into(),
            ],
            400,
            1,
        )
    }

    #[test]
    fn pokemon_edit_updates_health_and_moves() {
        let mut p = sample_pikachu();
        p.edit(vec!["Thunder".into(), "QuickAttack".into()], 250);
        assert_eq!(p.health, 250);
        assert_eq!(p.moves_name, vec!["Thunder", "QuickAttack"]);
        assert_eq!(p.name, "Pikachu");
    }

    #[test]
    fn apply_damage_normal_and_critical() {
        let mut p = sample_pikachu();
        p.apply_damage(90, false);
        assert_eq!(p.health, 310);

        p.apply_damage(90, true);
        assert_eq!(p.health, 200); // 310 - 90 - 20
    }

    #[test]
    fn is_fainted_when_health_depleted() {
        let mut p = sample_pikachu();
        assert!(!p.is_fainted());
        p.health = 0;
        assert!(p.is_fainted());
        p.health = -5;
        assert!(p.is_fainted());
    }

    #[test]
    fn health_check_returns_true_only_when_fainted() {
        let mut p = sample_pikachu();
        assert!(!p.health_check());
        p.health = 1;
        assert!(!p.health_check());
        p.health = 0;
        assert!(p.health_check());
    }

    #[test]
    fn take_hit_reduces_health_and_reports_damage() {
        let mut p = sample_pikachu();
        let before = p.health;
        let r = p.take_hit(1);
        assert!(!r.critical);
        assert!(!r.blocked);
        assert!(r.damage_dealt >= 80 && r.damage_dealt < 100);
        assert_eq!(p.health, before - r.damage_dealt);
    }

    #[test]
    fn health_ratio_clamps() {
        let mut p = sample_pikachu();
        assert!((p.health_ratio(400) - 1.0).abs() < f64::EPSILON);
        p.health = 200;
        assert!((p.health_ratio(400) - 0.5).abs() < f64::EPSILON);
        p.health = -10;
        assert!((p.health_ratio(400) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_roundtrip_preserves_pokemon_fields() {
        let team = PokemonsList {
            pokeball: vec![sample_pikachu()],
        };
        let json = serde_json::to_string(&team).expect("serialize");
        let back: PokemonsList = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, team);
        assert!(json.contains("\"moves\""));
        assert!(json.contains("\"type\""));
        assert!(json.contains("\"pokemons\""));
    }

    #[test]
    fn read_and_write_team_file_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "pokemon-text-game-test-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("team.json");

        let team = PokemonsList {
            pokeball: vec![
                sample_pikachu(),
                Pokemon::with_stats("Bulbasaur", vec!["VineWhip".into()], 450, 2),
            ],
        };

        write_team_to_file(&path, &team).expect("write");
        let loaded = read_team_from_file(&path).expect("read");
        assert_eq!(loaded, team);
        assert_eq!(loaded.pokeball.len(), 2);
        assert_eq!(loaded.pokeball[1].name, "Bulbasaur");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_bundled_pokemons_json_if_present() {
        let path = Path::new("resources/pokemons.json");
        if !path.exists() {
            return;
        }
        let team = read_team_from_file(path).expect("read resources/pokemons.json");
        assert!(!team.pokeball.is_empty());
        assert_eq!(team.pokeball[0].name, "Pikachu");
        assert!(team.pokeball[0].health > 0);
        assert!(!team.pokeball[0].moves_name.is_empty());
    }

    #[test]
    fn read_team_from_missing_file_errors() {
        let err = read_team_from_file("/nonexistent/pokemon-team-xyz.json");
        assert!(err.is_err());
    }

    #[test]
    fn read_team_rejects_invalid_json() {
        let dir = std::env::temp_dir().join(format!(
            "pokemon-text-game-bad-json-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("bad.json");
        {
            let mut f = File::create(&path).unwrap();
            writeln!(f, "{{ not valid json").unwrap();
        }
        let err = read_team_from_file(&path);
        assert!(err.is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_text_input_does_not_panic_on_parse() {
        assert!(parse_i64("none").is_err());
        assert!(!is_valid_choice(0, 3));
        assert!(parse_moves("a b").len() == 2);
    }
}
