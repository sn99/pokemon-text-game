/* MIT License — Copyright (c) 2018-2026 sn99 */

//! # Pokemon 2D Game — library crate
//!
//! Simplified Pokemon adventure:
//! - Species / moves / types (PokeAPI-compatible)
//! - Type-effective turn-based battles
//! - Overworld exploration with wild encounters
//! - Save / load

pub mod battle;
pub mod data;
pub mod game;
pub mod pokemon;
pub mod save;
pub mod world;

pub use battle::engine::{calculate_damage, AttackResult};
pub use pokemon::{ElementType, PokemonInstance};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_instance_and_type_chart_integrated() {
        let p = PokemonInstance::from_species_name("Charizard", 50).unwrap();
        assert_eq!(p.types.len(), 2);
        let mult = crate::pokemon::type_effectiveness(crate::pokemon::ElementType::Water, &p.types);
        assert!((mult - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn save_roundtrip() {
        let s = save::SaveGame::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: save::SaveGame = serde_json::from_str(&json).unwrap();
        assert_eq!(back.money, s.money);
    }
}
