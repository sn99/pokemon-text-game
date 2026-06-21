/*MIT License

Copyright (c) 2018-2026 sn99

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

//! # Pokemon Text Game — library crate
//!
//! Modular terminal Pokemon battler with:
//! - Species / moves / types database (PokeAPI-compatible shapes)
//! - Type-effective battle engine
//! - ASCII battle sprites
//! - Save games, inventory, wild encounters
//!
//! Binary entry point lives in `main.rs` (ratatui TUI).

pub mod ascii;
pub mod audio;
pub mod battle;
pub mod data;
pub mod extra;
pub mod pokemon;
pub mod save;
pub mod util;
pub mod world;

// ---- Backward-compatible re-exports (v1 API) ----

pub use battle::engine::{random_flavor as random_message_str, roll_block, roll_critical_chance, AttackResult};
pub use data::{read_team_from_file, write_team_to_file, TEAM_PATH};
pub use pokemon::{LegacyPokemon, Pokemon, PokemonInstance, PokemonsList};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extra::{is_valid_choice, parse_i64, parse_moves};
    use crate::pokemon::LegacyPokemon;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

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
        assert_eq!(p.health, 200);
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
                LegacyPokemon::with_stats("Bulbasaur", vec!["VineWhip".into()], 450, 2),
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

    #[test]
    fn v2_instance_and_type_chart_integrated() {
        let p = PokemonInstance::from_species_name("Charizard", 50).unwrap();
        assert_eq!(p.types.len(), 2);
        let mult = crate::pokemon::type_effectiveness(
            crate::pokemon::ElementType::Water,
            &p.types,
        );
        assert!((mult - 2.0).abs() < f64::EPSILON); // Water vs Fire/Flying = 2 * 1? Wait Flying is neutral to water = 2
        assert!((mult - 2.0).abs() < f64::EPSILON);
    }
}
