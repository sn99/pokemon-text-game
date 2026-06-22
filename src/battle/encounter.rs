//! Wild encounters, starters, gym/trainer/elite teams, exhibition foes.

use rand::Rng;

use crate::pokemon::db::all_species;
use crate::pokemon::species::species_by_id;
use crate::pokemon::PokemonInstance;
use crate::world::{
    default_gyms, default_routes, elite_four, rival_team, route_trainers, EliteTrainer, GymLeader,
    Route, RouteTrainer,
};

pub fn starter_party(choice: usize) -> Vec<PokemonInstance> {
    let ids = [1u16, 4, 7];
    let id = ids[choice % 3];
    let main = PokemonInstance::from_species_id(id, 5).expect("starter");
    let mut party = vec![main];
    if let Some(pika) = PokemonInstance::from_species_id(25, 5) {
        party.push(pika);
    }
    party
}

pub fn random_wild_encounter(min_level: u8, max_level: u8) -> PokemonInstance {
    let species = all_species();
    let wild: Vec<_> = species.iter().filter(|s| s.id < 900 && s.id > 0).collect();
    let mut rng = rand::thread_rng();
    let s = wild[rng.gen_range(0..wild.len())];
    let lo = min_level.min(max_level);
    let hi = min_level.max(max_level);
    let level = rng.gen_range(lo..=hi);
    PokemonInstance::from_species(s, level)
}

pub fn wild_on_route(route: &Route) -> PokemonInstance {
    let mut rng = rand::thread_rng();
    let id = if route.spawn_pool.is_empty() {
        let all = all_species();
        all[rng.gen_range(0..all.len())].id
    } else {
        route.spawn_pool[rng.gen_range(0..route.spawn_pool.len())]
    };
    let level = rng.gen_range(route.min_level..=route.max_level);
    PokemonInstance::from_species_id(id, level)
        .unwrap_or_else(|| random_wild_encounter(route.min_level, route.max_level))
}

/// Fishing / surf encounter using water_pool.
pub fn water_on_route(route: &Route) -> Option<PokemonInstance> {
    if route.water_pool.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let id = route.water_pool[rng.gen_range(0..route.water_pool.len())];
    let level = rng.gen_range(route.min_level..=route.max_level);
    PokemonInstance::from_species_id(id, level)
}

pub fn route_by_id(id: u8) -> Option<Route> {
    default_routes().into_iter().find(|r| r.id == id)
}

pub fn gym_by_index(i: usize) -> Option<GymLeader> {
    default_gyms().into_iter().nth(i)
}

pub fn gym_lead_instance(gym: &GymLeader) -> PokemonInstance {
    let sid = gym.team_ids.first().copied().unwrap_or(74);
    PokemonInstance::from_species_id(sid, gym.level)
        .unwrap_or_else(|| random_wild_encounter(gym.level, gym.level))
}

pub fn trainer_from_ids(ids: &[u16], level: u8) -> PokemonInstance {
    let sid = ids.first().copied().unwrap_or(16);
    PokemonInstance::from_species_id(sid, level)
        .unwrap_or_else(|| random_wild_encounter(level, level))
}

pub fn route_trainer_instance(t: &RouteTrainer) -> PokemonInstance {
    trainer_from_ids(&t.team_ids, t.level)
}

pub fn elite_lead_instance(e: &EliteTrainer) -> PokemonInstance {
    trainer_from_ids(&e.team_ids, e.level)
}

pub fn rival_instance(stage: u8, starter_line: u16) -> PokemonInstance {
    let (_name, level, team) = rival_team(stage, starter_line);
    trainer_from_ids(&team, level)
}

pub fn exhibition_foe(species_id: u16, level: u8) -> Option<PokemonInstance> {
    species_by_id(species_id).map(|s| PokemonInstance::from_species(&s, level))
}

pub fn trainers_on_route(route_id: u8) -> Vec<RouteTrainer> {
    route_trainers()
        .into_iter()
        .filter(|t| t.route_id == route_id)
        .collect()
}

pub fn elite_by_index(i: usize) -> Option<EliteTrainer> {
    elite_four().into_iter().nth(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_spawn_works() {
        let r = route_by_id(1).unwrap();
        let w = wild_on_route(&r);
        assert!(w.max_hp > 0);
    }

    #[test]
    fn exhibition_pikachu() {
        let p = exhibition_foe(25, 20).unwrap();
        assert_eq!(p.species_id, 25);
    }

    #[test]
    fn starter_party_has_two() {
        assert_eq!(starter_party(0).len(), 2);
    }

    #[test]
    fn water_coast() {
        let r = route_by_id(4).unwrap();
        assert!(water_on_route(&r).is_some());
    }
}
