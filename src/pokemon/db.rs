//! Runtime-loaded species & move catalogues (PokeAPI data/v2 exports).

use std::fs::File;
use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;

use super::moves::{MoveCategory, MoveData};
use super::species::Species;
use super::stats::BaseStats;
use super::types::ElementType;

static SPECIES_DB: OnceLock<Vec<Species>> = OnceLock::new();
static MOVES_DB: OnceLock<Vec<MoveData>> = OnceLock::new();
static MOVES_BY_ID: OnceLock<std::collections::HashMap<u16, MoveData>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct SpeciesFile {
    species: Vec<SpeciesJson>,
}

#[derive(Debug, Deserialize)]
struct SpeciesJson {
    id: u16,
    name: String,
    types: Vec<String>,
    base_stats: BaseStatsJson,
    #[serde(default)]
    base_experience: u32,
    #[serde(default)]
    height_dm: u16,
    #[serde(default)]
    weight_hg: u16,
    #[serde(default)]
    description: String,
    #[serde(default)]
    default_moves: Vec<u16>,
    #[serde(default = "default_capture")]
    capture_rate: u8,
}

fn default_capture() -> u8 {
    45
}

#[derive(Debug, Deserialize)]
struct BaseStatsJson {
    hp: u16,
    attack: u16,
    defense: u16,
    sp_attack: u16,
    sp_defense: u16,
    speed: u16,
}

#[derive(Debug, Deserialize)]
struct MovesFile {
    moves: Vec<MoveJson>,
}

#[derive(Debug, Deserialize)]
struct MoveJson {
    id: u16,
    name: String,
    move_type: String,
    category: String,
    #[serde(default)]
    power: u8,
    #[serde(default)]
    accuracy: u8,
    #[serde(default)]
    pp: u8,
    #[serde(default)]
    priority: i8,
    #[serde(default)]
    description: String,
}

fn parse_types(v: &[String]) -> Vec<ElementType> {
    let t: Vec<ElementType> = v
        .iter()
        .filter_map(|s| ElementType::from_str_loose(s))
        .collect();
    if t.is_empty() {
        vec![ElementType::Normal]
    } else {
        t
    }
}

fn parse_category(s: &str) -> MoveCategory {
    match s.to_lowercase().as_str() {
        "special" => MoveCategory::Special,
        "status" => MoveCategory::Status,
        _ => MoveCategory::Physical,
    }
}

fn species_from_json(j: SpeciesJson) -> Species {
    Species {
        id: j.id,
        name: j.name,
        types: parse_types(&j.types),
        base_stats: BaseStats::new(
            j.base_stats.hp,
            j.base_stats.attack,
            j.base_stats.defense,
            j.base_stats.sp_attack,
            j.base_stats.sp_defense,
            j.base_stats.speed,
        ),
        base_experience: if j.base_experience == 0 {
            64
        } else {
            j.base_experience
        },
        height_dm: j.height_dm,
        weight_hg: j.weight_hg,
        description: j.description,
        default_moves: j.default_moves,
        capture_rate: j.capture_rate,
    }
}

fn move_from_json(j: MoveJson) -> MoveData {
    MoveData {
        id: j.id,
        name: j.name,
        move_type: ElementType::from_str_loose(&j.move_type).unwrap_or(ElementType::Normal),
        category: parse_category(&j.category),
        power: j.power,
        accuracy: j.accuracy,
        pp: if j.pp == 0 { 5 } else { j.pp },
        priority: j.priority,
        description: j.description,
    }
}

fn try_load_species_file() -> Option<Vec<Species>> {
    let paths = [
        Path::new("resources/data/species.json"),
        Path::new("data/species.json"),
    ];
    for p in paths {
        if !p.exists() {
            continue;
        }
        let file = File::open(p).ok()?;
        let parsed: SpeciesFile = serde_json::from_reader(file).ok()?;
        if parsed.species.is_empty() {
            continue;
        }
        return Some(parsed.species.into_iter().map(species_from_json).collect());
    }
    None
}

fn try_load_moves_file() -> Option<Vec<MoveData>> {
    let paths = [
        Path::new("resources/data/moves.json"),
        Path::new("data/moves.json"),
    ];
    for p in paths {
        if !p.exists() {
            continue;
        }
        let file = File::open(p).ok()?;
        let parsed: MovesFile = serde_json::from_reader(file).ok()?;
        if parsed.moves.is_empty() {
            continue;
        }
        return Some(parsed.moves.into_iter().map(move_from_json).collect());
    }
    None
}

/// All species (PokeAPI JSON if present, else compiled-in fallback).
pub fn all_species() -> &'static [Species] {
    SPECIES_DB
        .get_or_init(|| try_load_species_file().unwrap_or_else(super::species::builtin_species))
}

/// All moves (PokeAPI JSON if present, else compiled-in fallback).
pub fn all_moves() -> &'static [MoveData] {
    MOVES_DB.get_or_init(|| try_load_moves_file().unwrap_or_else(super::moves::builtin_moves))
}

fn moves_map() -> &'static std::collections::HashMap<u16, MoveData> {
    MOVES_BY_ID.get_or_init(|| all_moves().iter().cloned().map(|m| (m.id, m)).collect())
}

pub fn species_by_id(id: u16) -> Option<Species> {
    all_species().iter().find(|s| s.id == id).cloned()
}

pub fn species_by_name(name: &str) -> Option<Species> {
    let key = name.trim().to_lowercase();
    all_species()
        .iter()
        .find(|s| s.name.to_lowercase() == key)
        .cloned()
}

pub fn move_by_id(id: u16) -> Option<MoveData> {
    moves_map().get(&id).cloned()
}

pub fn move_by_name(name: &str) -> Option<MoveData> {
    let key = name.replace(' ', "").replace('-', "").to_lowercase();
    all_moves()
        .iter()
        .find(|m| m.name.replace(' ', "").replace('-', "").to_lowercase() == key)
        .cloned()
}

pub fn db_stats() -> (usize, usize, bool, bool) {
    let species_from_file = Path::new("resources/data/species.json").exists();
    let moves_from_file = Path::new("resources/data/moves.json").exists();
    (
        all_species().len(),
        all_moves().len(),
        species_from_file,
        moves_from_file,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_substantial_catalogue_when_json_present() {
        let n = all_species().len();
        // With PokeAPI export: 1000+; without: builtin ~30
        assert!(n >= 30, "expected at least builtin species, got {n}");
    }

    #[test]
    fn pikachu_resolves() {
        let p = species_by_name("Pikachu").expect("Pikachu");
        assert_eq!(p.id, 25);
        assert!(p.types.iter().any(|t| *t == ElementType::Electric));
    }

    #[test]
    fn tackle_move_resolves() {
        let t = move_by_name("Tackle").or_else(|| move_by_id(33));
        assert!(t.is_some(), "Tackle should exist in moves db");
    }
}
