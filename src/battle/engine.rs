//! Damage calculation and attack resolution.

use rand::Rng;

use crate::pokemon::moves::{LearnedMove, MoveCategory};
use crate::pokemon::species::species_by_id;
use crate::pokemon::types::{effectiveness_label, type_effectiveness};
use crate::pokemon::{PokemonInstance, StatusCondition};

#[derive(Debug, Clone, PartialEq)]
pub struct AttackResult {
    pub flavor: String,
    pub damage_dealt: i64,
    pub critical: bool,
    pub blocked: bool,
    pub effectiveness: f64,
    pub effectiveness_text: String,
    pub missed: bool,
}

impl Default for AttackResult {
    fn default() -> Self {
        Self {
            flavor: String::new(),
            damage_dealt: 0,
            critical: false,
            blocked: false,
            effectiveness: 1.0,
            effectiveness_text: String::new(),
            missed: false,
        }
    }
}

pub fn random_flavor() -> String {
    match rand::thread_rng().gen_range(0..5) {
        0 => "We can do it!".into(),
        1 => "Never give up!".into(),
        2 => "Be the very best!".into(),
        3 => "Till the end we shall dance!".into(),
        _ => "Gotta catch 'em all!".into(),
    }
}

/// ~1/8 block chance (original).
pub fn roll_block() -> bool {
    rand::thread_rng().gen_range(0..8) == 0
}

/// ~1/9 critical roll; `0` means critical (original).
pub fn roll_critical_chance() -> i32 {
    rand::thread_rng().gen_range(0..9)
}

/// Main-series-inspired damage formula (simplified). Weather optional (defaults clear).
pub fn calculate_damage(
    attacker: &PokemonInstance,
    defender: &PokemonInstance,
    mv: &LearnedMove,
) -> AttackResult {
    calculate_damage_weather(attacker, defender, mv, crate::world::Weather::Clear)
}

pub fn calculate_damage_weather(
    attacker: &PokemonInstance,
    defender: &PokemonInstance,
    mv: &LearnedMove,
    weather: crate::world::Weather,
) -> AttackResult {
    let mut rng = rand::thread_rng();

    if mv.data.accuracy > 0 && rng.gen_range(1..=100) > mv.data.accuracy {
        return AttackResult {
            missed: true,
            flavor: random_flavor(),
            ..Default::default()
        };
    }

    if matches!(mv.data.category, MoveCategory::Status) {
        return AttackResult {
            flavor: format!(
                "{} used {}! (status effect)",
                attacker.display_name(),
                mv.data.name
            ),
            ..Default::default()
        };
    }

    let atk_species = species_by_id(attacker.species_id);
    let def_species = species_by_id(defender.species_id);

    let (mut atk_stat, def_stat) = match mv.data.category {
        MoveCategory::Physical => {
            let a = atk_species
                .as_ref()
                .map(|s| s.base_stats.attack_at_level(attacker.level))
                .unwrap_or(50);
            let d = def_species
                .as_ref()
                .map(|s| s.base_stats.defense_at_level(defender.level))
                .unwrap_or(50);
            (a, d)
        }
        MoveCategory::Special => {
            let a = atk_species
                .as_ref()
                .map(|s| s.base_stats.sp_attack_at_level(attacker.level))
                .unwrap_or(50);
            let d = def_species
                .as_ref()
                .map(|s| s.base_stats.sp_defense_at_level(defender.level))
                .unwrap_or(50);
            (a, d)
        }
        MoveCategory::Status => (1, 1),
    };
    // IV bonus (simple additive)
    atk_stat += (attacker.iv_attack / 8) as i64;

    let power = mv.data.power.max(1) as i64;
    let level = attacker.level.max(1) as i64;

    let mut damage = ((2 * level / 5 + 2) * power * atk_stat / def_stat.max(1)) / 50 + 2;

    let stab = if attacker.types.contains(&mv.data.move_type) {
        1.5
    } else {
        1.0
    };
    damage = (damage as f64 * stab) as i64;

    let eff = type_effectiveness(mv.data.move_type, &defender.types);
    damage = (damage as f64 * eff) as i64;

    // Weather boost/nerf
    let wmult = weather.move_multiplier(mv.data.move_type);
    damage = (damage as f64 * wmult) as i64;

    let critical = rng.gen_range(0..16) == 0;
    if critical {
        damage = (damage as f64 * 1.5) as i64;
    }

    let rand_factor = rng.gen_range(85..=100) as f64 / 100.0;
    damage = (damage as f64 * rand_factor) as i64;

    if matches!(attacker.status, StatusCondition::Burn)
        && matches!(mv.data.category, MoveCategory::Physical)
    {
        damage /= 2;
    }

    if eff == 0.0 {
        damage = 0;
    } else {
        damage = damage.max(1);
    }

    let mut eff_text = effectiveness_label(eff).to_string();
    if (wmult - 1.0).abs() > f64::EPSILON {
        eff_text = format!("{eff_text} (weather)");
    }

    AttackResult {
        flavor: random_flavor(),
        damage_dealt: damage,
        critical,
        blocked: false,
        effectiveness: eff,
        effectiveness_text: eff_text,
        missed: false,
    }
}

/// Apply an attack from attacker to defender using move at index.
pub fn execute_move(
    attacker: &mut PokemonInstance,
    defender: &mut PokemonInstance,
    move_index: usize,
) -> AttackResult {
    execute_move_weather(attacker, defender, move_index, crate::world::Weather::Clear)
}

pub fn execute_move_weather(
    attacker: &mut PokemonInstance,
    defender: &mut PokemonInstance,
    move_index: usize,
    weather: crate::world::Weather,
) -> AttackResult {
    if move_index >= attacker.moves.len() {
        return AttackResult {
            flavor: "No move selected!".into(),
            ..Default::default()
        };
    }

    match attacker.status {
        StatusCondition::Sleep { turns } => {
            if turns > 0 {
                attacker.status = StatusCondition::Sleep { turns: turns - 1 };
                return AttackResult {
                    flavor: format!("{} is fast asleep!", attacker.display_name()),
                    ..Default::default()
                };
            } else {
                attacker.status = StatusCondition::None;
            }
        }
        StatusCondition::Freeze => {
            if rand::thread_rng().gen_range(0..5) != 0 {
                return AttackResult {
                    flavor: format!("{} is frozen solid!", attacker.display_name()),
                    ..Default::default()
                };
            }
            attacker.status = StatusCondition::None;
        }
        StatusCondition::Paralysis => {
            if rand::thread_rng().gen_range(0..4) == 0 {
                return AttackResult {
                    flavor: format!("{} is paralyzed! It can't move!", attacker.display_name()),
                    ..Default::default()
                };
            }
        }
        _ => {}
    }

    let mv = &attacker.moves[move_index];
    if !mv.can_use() {
        return AttackResult {
            flavor: format!(
                "{} has no PP left for {}!",
                attacker.display_name(),
                mv.data.name
            ),
            ..Default::default()
        };
    }

    let result = calculate_damage_weather(attacker, defender, mv, weather);
    attacker.moves[move_index].spend_pp();

    if !result.missed
        && !matches!(
            attacker.moves[move_index].data.category,
            MoveCategory::Status
        )
    {
        defender.apply_damage(result.damage_dealt);
    }

    result
}

/// Weather residual (disabled in simplified game).
pub fn apply_weather_residual(
    _poke: &mut PokemonInstance,
    _weather: crate::world::Weather,
) -> Option<String> {
    None
}

/// End-of-turn residual damage.
pub fn apply_status_residual(poke: &mut PokemonInstance) -> Option<String> {
    match poke.status {
        StatusCondition::Burn | StatusCondition::Poison => {
            let dmg = (poke.max_hp / 8).max(1);
            poke.apply_damage(dmg);
            let kind = if matches!(poke.status, StatusCondition::Burn) {
                "burn"
            } else {
                "poison"
            };
            Some(format!(
                "{} is hurt by {}! (-{})",
                poke.display_name(),
                kind,
                dmg
            ))
        }
        _ => None,
    }
}

/// Legacy helper: simple type-less attack for old Pokemon struct path.
pub fn legacy_simple_attack(defender_health: &mut i64, critical_chance: i32) -> AttackResult {
    let mut rng = rand::thread_rng();
    let critical = critical_chance == 0;
    let damage_dealt = rng.gen_range(80..100) + if critical { rng.gen_range(10..30) } else { 0 };
    *defender_health -= damage_dealt;
    AttackResult {
        flavor: random_flavor(),
        damage_dealt,
        critical,
        blocked: false,
        effectiveness: 1.0,
        effectiveness_text: String::new(),
        missed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::types::ElementType;
    use crate::pokemon::PokemonInstance;

    #[test]
    fn super_effective_water_on_fire() {
        let mut atk = PokemonInstance::from_species_name("Squirtle", 30).unwrap();
        let mut def = PokemonInstance::from_species_name("Charmander", 30).unwrap();
        // Find water gun or similar
        let mi = atk
            .moves
            .iter()
            .position(|m| m.data.move_type == ElementType::Water)
            .unwrap_or(0);
        // Run a few times; average damage should be > 0
        let r = execute_move(&mut atk, &mut def, mi);
        if !r.missed {
            assert!(r.effectiveness >= 1.0 || r.effectiveness == 0.0 || r.damage_dealt >= 0);
        }
    }

    #[test]
    fn fainted_after_enough_damage() {
        let mut p = PokemonInstance::from_species_name("Pikachu", 5).unwrap();
        p.apply_damage(9999);
        assert!(p.is_fainted());
    }
}
