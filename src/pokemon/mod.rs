//! Pokemon domain: types, stats, moves, species, and runtime instances.

pub mod db;
pub mod instance;
pub mod moves;
pub mod species;
pub mod stats;
pub mod types;

pub use db::{all_moves, all_species, db_stats};
pub use instance::{
    LegacyPokemon, Nature, Pokemon, PokemonInstance, PokemonsList, StatusCondition,
};
pub use moves::{
    builtin_moves, learned_from_legacy_names, move_by_id, move_by_name, LearnedMove, MoveCategory,
    MoveData,
};
pub use species::{builtin_species, species_by_id, species_by_name, Species};
pub use stats::{xp_for_level, xp_gain_from_defeat, BaseStats};
pub use types::{effectiveness_label, type_effectiveness, ElementType};
