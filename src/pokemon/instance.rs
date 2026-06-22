//! Runtime Pokemon instance (party / battle unit).

use serde::{Deserialize, Serialize};

use super::moves::{learned_from_legacy_names, move_by_id, LearnedMove};
use super::species::{species_by_id, species_by_name, Species};
use super::stats::{xp_for_level, xp_gain_from_defeat};
use super::types::ElementType;

/// Legacy team format (resources/pokemons.json) for backward compatibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PokemonsList {
    #[serde(rename = "pokemons")]
    pub pokeball: Vec<LegacyPokemon>,
}

/// Old-style Pokemon record kept for serde compatibility with existing saves.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyPokemon {
    pub name: String,
    #[serde(rename = "moves")]
    pub moves_name: Vec<String>,
    pub health: i64,
    #[serde(rename = "type")]
    pub pokemon_type: i64,
}

impl LegacyPokemon {
    pub fn with_stats(
        name: impl Into<String>,
        moves_name: Vec<String>,
        health: i64,
        pokemon_type: i64,
    ) -> Self {
        Self {
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

    pub fn is_fainted(&self) -> bool {
        self.health <= 0
    }

    pub fn health_check(&self) -> bool {
        self.is_fainted()
    }

    pub fn health_ratio(&self, max_health: i64) -> f64 {
        if max_health <= 0 {
            return 0.0;
        }
        (self.health.max(0) as f64 / max_health as f64).clamp(0.0, 1.0)
    }

    pub fn apply_damage(&mut self, amount: i64, critical: bool) {
        self.health -= amount + if critical { 20 } else { 0 };
    }

    /// Simple randomized hit used by legacy battle path.
    pub fn take_hit(&mut self, critical_chance: i32) -> crate::battle::engine::AttackResult {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let critical = critical_chance == 0;
        let damage_dealt =
            rng.gen_range(80..100) + if critical { rng.gen_range(10..30) } else { 0 };
        self.health -= damage_dealt;
        crate::battle::engine::AttackResult {
            flavor: crate::battle::engine::random_flavor(),
            damage_dealt,
            critical,
            blocked: false,
            effectiveness: 1.0,
            effectiveness_text: String::new(),
            missed: false,
        }
    }

    pub fn to_instance(&self) -> PokemonInstance {
        let species = species_by_name(&self.name);
        let level = 25u8;
        let types = if let Some(ref s) = species {
            s.types.clone()
        } else {
            vec![ElementType::from_legacy_id(self.pokemon_type)]
        };
        let species_id = species.as_ref().map(|s| s.id).unwrap_or(0);
        let max_hp = self.health.max(1);
        let moves = learned_from_legacy_names(&self.moves_name);
        PokemonInstance {
            species_id,
            nickname: self.name.clone(),
            level,
            current_hp: self.health,
            max_hp,
            types,
            moves,
            experience: 0,
            status: StatusCondition::None,
            nature: Nature::Hardy,
            iv_attack: 15,
            iv_speed: 15,
            shiny: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Nature {
    #[default]
    Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky,
}

impl Nature {
    pub fn all() -> &'static [Nature] {
        use Nature::*;
        &[
            Hardy, Lonely, Brave, Adamant, Naughty, Bold, Docile, Relaxed, Impish, Lax, Timid,
            Hasty, Serious, Jolly, Naive, Modest, Mild, Quiet, Bashful, Rash, Calm, Gentle, Sassy,
            Careful, Quirky,
        ]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Hardy => "Hardy",
            Self::Lonely => "Lonely",
            Self::Brave => "Brave",
            Self::Adamant => "Adamant",
            Self::Naughty => "Naughty",
            Self::Bold => "Bold",
            Self::Docile => "Docile",
            Self::Relaxed => "Relaxed",
            Self::Impish => "Impish",
            Self::Lax => "Lax",
            Self::Timid => "Timid",
            Self::Hasty => "Hasty",
            Self::Serious => "Serious",
            Self::Jolly => "Jolly",
            Self::Naive => "Naive",
            Self::Modest => "Modest",
            Self::Mild => "Mild",
            Self::Quiet => "Quiet",
            Self::Bashful => "Bashful",
            Self::Rash => "Rash",
            Self::Calm => "Calm",
            Self::Gentle => "Gentle",
            Self::Sassy => "Sassy",
            Self::Careful => "Careful",
            Self::Quirky => "Quirky",
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        let all = Self::all();
        all[rand::thread_rng().gen_range(0..all.len())]
    }
}

/// Type alias preserved for external/old code that referred to `Pokemon`.
pub type Pokemon = LegacyPokemon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StatusCondition {
    #[default]
    None,
    Burn,
    Poison,
    Paralysis,
    Sleep {
        turns: u8,
    },
    Freeze,
}

impl StatusCondition {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Burn => "BRN",
            Self::Poison => "PSN",
            Self::Paralysis => "PAR",
            Self::Sleep { .. } => "SLP",
            Self::Freeze => "FRZ",
        }
    }
}

/// Fully-featured battle/party Pokemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PokemonInstance {
    pub species_id: u16,
    pub nickname: String,
    pub level: u8,
    pub current_hp: i64,
    pub max_hp: i64,
    pub types: Vec<ElementType>,
    pub moves: Vec<LearnedMove>,
    pub experience: u32,
    pub status: StatusCondition,
    #[serde(default)]
    pub nature: Nature,
    #[serde(default = "default_iv")]
    pub iv_attack: u8,
    #[serde(default = "default_iv")]
    pub iv_speed: u8,
    #[serde(default)]
    pub shiny: bool,
}

fn default_iv() -> u8 {
    15
}

impl PokemonInstance {
    pub fn from_species(species: &Species, level: u8) -> Self {
        use rand::Rng;
        let level = level.clamp(1, 100);
        let max_hp = species.base_stats.max_hp_at_level(level);
        let moves: Vec<LearnedMove> = species
            .default_moves
            .iter()
            .filter_map(|&id| move_by_id(id))
            .map(LearnedMove::new)
            .collect();
        let moves = if moves.is_empty() {
            learned_from_legacy_names(&["Tackle".into()])
        } else {
            moves
        };
        let mut rng = rand::thread_rng();
        // ~1/512 shiny rate
        let shiny = rng.gen_range(0..512) == 0;
        Self {
            species_id: species.id,
            nickname: species.name.clone(),
            level,
            current_hp: max_hp,
            max_hp,
            types: species.types.clone(),
            moves,
            experience: 0,
            status: StatusCondition::None,
            nature: Nature::random(),
            iv_attack: rng.gen_range(0..=31),
            iv_speed: rng.gen_range(0..=31),
            shiny,
        }
    }

    pub fn from_species_id(id: u16, level: u8) -> Option<Self> {
        species_by_id(id).map(|s| Self::from_species(&s, level))
    }

    pub fn from_species_name(name: &str, level: u8) -> Option<Self> {
        species_by_name(name).map(|s| Self::from_species(&s, level))
    }

    pub fn display_name(&self) -> &str {
        &self.nickname
    }

    pub fn set_nickname(&mut self, name: impl Into<String>) {
        let n = name.into().trim().to_string();
        if !n.is_empty() {
            self.nickname = n.chars().take(12).collect();
        }
    }

    pub fn shiny_prefix(&self) -> &'static str {
        if self.shiny {
            "✦"
        } else {
            ""
        }
    }

    pub fn is_fainted(&self) -> bool {
        self.current_hp <= 0
    }

    /// Teach first available species default move not already known.
    pub fn tutor_relearn_move(&mut self) -> Result<String, String> {
        let Some(sp) = species_by_id(self.species_id) else {
            return Err("Unknown species.".into());
        };
        for &mid in &sp.default_moves {
            let Some(mv) = move_by_id(mid) else { continue };
            if self.moves.iter().any(|m| m.data.id == mv.id) {
                continue;
            }
            let name = mv.name.clone();
            if self.moves.len() < 4 {
                self.moves.push(LearnedMove::new(mv));
            } else {
                // Replace weakest (lowest power) move
                if let Some((i, _)) = self
                    .moves
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, m)| m.data.power)
                {
                    self.moves[i] = LearnedMove::new(mv);
                }
            }
            return Ok(format!("{} learned {}!", self.display_name(), name));
        }
        Err("No new moves to teach.".into())
    }

    /// Attempt level-up evolution; returns messages if evolved.
    pub fn try_evolve(&mut self) -> Vec<String> {
        use crate::world::evolution_for;
        let mut msgs = Vec::new();
        if let Some(to_id) = evolution_for(self.species_id, self.level) {
            if let Some(sp) = species_by_id(to_id) {
                let old_name = self.nickname.clone();
                let ratio = if self.max_hp > 0 {
                    self.current_hp as f64 / self.max_hp as f64
                } else {
                    1.0
                };
                let was_default_name = species_by_id(self.species_id)
                    .map(|s| s.name == old_name)
                    .unwrap_or(false);
                self.species_id = to_id;
                self.types = sp.types.clone();
                self.max_hp = sp.base_stats.max_hp_at_level(self.level);
                self.current_hp = (self.max_hp as f64 * ratio).round() as i64;
                if was_default_name {
                    self.nickname = sp.name.clone();
                }
                msgs.push(format!("What? {old_name} is evolving!"));
                msgs.push(format!(
                    "{} evolved into {}!",
                    if was_default_name {
                        &sp.name
                    } else {
                        &old_name
                    },
                    sp.name
                ));
            }
        }
        msgs
    }

    pub fn health_ratio(&self) -> f64 {
        if self.max_hp <= 0 {
            return 0.0;
        }
        (self.current_hp.max(0) as f64 / self.max_hp as f64).clamp(0.0, 1.0)
    }

    pub fn heal_full(&mut self) {
        self.current_hp = self.max_hp;
        self.status = StatusCondition::None;
        for m in &mut self.moves {
            m.current_pp = m.data.pp;
        }
    }

    pub fn apply_damage(&mut self, amount: i64) {
        self.current_hp = (self.current_hp - amount).max(0);
    }

    pub fn heal(&mut self, amount: i64) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn primary_type(&self) -> ElementType {
        self.types.first().copied().unwrap_or(ElementType::Normal)
    }

    pub fn gain_xp(&mut self, amount: u32) -> Vec<String> {
        let mut msgs = Vec::new();
        self.experience += amount;
        msgs.push(format!("{} gained {} XP!", self.nickname, amount));
        while self.level < 100 {
            let need = xp_for_level(self.level + 1);
            if self.experience >= need {
                self.level += 1;
                // Recompute HP proportionally
                if let Some(sp) = species_by_id(self.species_id) {
                    let old_max = self.max_hp;
                    let ratio = if old_max > 0 {
                        self.current_hp as f64 / old_max as f64
                    } else {
                        1.0
                    };
                    self.max_hp = sp.base_stats.max_hp_at_level(self.level);
                    self.current_hp = (self.max_hp as f64 * ratio).round() as i64;
                }
                msgs.push(format!("{} grew to level {}!", self.nickname, self.level));
                msgs.extend(self.try_evolve());
            } else {
                break;
            }
        }
        msgs
    }

    pub fn xp_for_defeating(&self, base_yield: u32) -> u32 {
        xp_gain_from_defeat(self.level, base_yield)
    }

    pub fn to_legacy(&self) -> LegacyPokemon {
        LegacyPokemon {
            name: self.nickname.clone(),
            moves_name: self.moves.iter().map(|m| m.data.name.clone()).collect(),
            health: self.current_hp.max(1),
            pokemon_type: match self.primary_type() {
                ElementType::Electric => 1,
                ElementType::Grass => 2,
                ElementType::Water => 3,
                ElementType::Fire => 4,
                ElementType::Psychic => 6,
                ElementType::Fighting => 7,
                ElementType::Ghost => 8,
                ElementType::Dragon => 9,
                _ => 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pikachu_instance_from_species() {
        let p = PokemonInstance::from_species_name("Pikachu", 20).unwrap();
        assert_eq!(p.nickname, "Pikachu");
        assert!(p.max_hp > 0);
        assert!(!p.moves.is_empty());
        assert!(!p.is_fainted());
    }

    #[test]
    fn damage_and_faint() {
        let mut p = PokemonInstance::from_species_name("Pikachu", 10).unwrap();
        p.apply_damage(p.max_hp + 10);
        assert!(p.is_fainted());
    }

    #[test]
    fn legacy_roundtrip_fields() {
        let leg = LegacyPokemon::with_stats("Pikachu", vec!["Tackle".into()], 400, 1);
        let inst = leg.to_instance();
        assert_eq!(inst.nickname, "Pikachu");
        assert!(!inst.moves.is_empty());
    }

    #[test]
    fn heal_full_restores_pp_and_hp() {
        let mut p = PokemonInstance::from_species_name("Squirtle", 15).unwrap();
        p.current_hp = 1;
        if let Some(m) = p.moves.first_mut() {
            m.current_pp = 0;
        }
        p.heal_full();
        assert_eq!(p.current_hp, p.max_hp);
        assert!(p.moves.iter().all(|m| m.current_pp == m.data.pp));
    }
}
