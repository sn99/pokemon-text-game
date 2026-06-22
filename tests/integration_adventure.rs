//! Integration tests for simplified game core.

use pokemon_text_game::battle::execute_move;
use pokemon_text_game::pokemon::PokemonInstance;
use pokemon_text_game::save::{roll_capture, DexProgress, ItemKind, SaveGame, DEX_SIZE};
use pokemon_text_game::world::{
    area_name_for_tile, build_map, evolution_for, interact_prop_at, is_npc_at, starter_species,
    world_props, PropKind, Tile,
};

#[test]
fn starters_exist() {
    for (id, name) in starter_species() {
        let p = PokemonInstance::from_species_id(id, 5).expect("starter");
        assert_eq!(p.species_id, id);
        assert!(p.display_name().contains(name) || p.nickname == name || !p.nickname.is_empty());
        assert!(p.max_hp > 0);
        assert!(!p.moves.is_empty());
    }
}

#[test]
fn map_has_tall_grass_and_center() {
    let m = build_map();
    let mut tall = 0;
    let mut door = 0;
    let mut sand = 0;
    let mut sign = 0;
    for row in &m {
        for t in row {
            if *t == Tile::TallGrass {
                tall += 1;
            }
            if *t == Tile::Door {
                door += 1;
            }
            if *t == Tile::Sand {
                sand += 1;
            }
            if *t == Tile::Sign {
                sign += 1;
            }
        }
    }
    assert!(tall > 10, "expected tall grass patches");
    assert_eq!(door, 1, "expected one center door");
    assert!(sand > 0, "expected sandy shore near pond");
    assert!(sign >= 3, "expected sign tiles under route signs");
}

#[test]
fn battle_damages_foe() {
    let mut a = PokemonInstance::from_species_id(4, 20).unwrap();
    let mut b = PokemonInstance::from_species_id(7, 20).unwrap();
    let before = b.current_hp;
    let _ = execute_move(&mut a, &mut b, 0);
    assert!(b.current_hp <= before);
}

#[test]
fn save_items_and_heal() {
    let mut s = SaveGame::default();
    assert_eq!(s.item_count(ItemKind::PokeBall), 8);
    s.party
        .push(PokemonInstance::from_species_id(1, 5).unwrap());
    s.party[0].current_hp = 1;
    s.heal_party_full();
    assert_eq!(s.party[0].current_hp, s.party[0].max_hp);
}

#[test]
fn capture_roll_runs() {
    let _ = roll_capture(100, 1, 255, 1.0);
    let _ = roll_capture(100, 100, 3, 1.0);
}

#[test]
fn bulbasaur_evolves_at_16() {
    assert_eq!(evolution_for(1, 15), None);
    assert_eq!(evolution_for(1, 16), Some(2));
}

#[test]
fn area_names_and_mart_tiles_exist() {
    let m = build_map();
    let mut mart = 0;
    for row in &m {
        for t in row {
            if matches!(t, Tile::Mart | Tile::MartDoor) {
                mart += 1;
            }
        }
    }
    assert!(mart >= 2, "mart tiles missing");
    assert_eq!(area_name_for_tile(7, 16), "Pallet Town");
    assert_eq!(area_name_for_tile(22, 19), "South Pond");
    // Pond priority: water/shore stays South Pond; tile north of pond band may be east route
    assert_eq!(area_name_for_tile(23, 18), "South Pond");
    assert_eq!(area_name_for_tile(23, 10), "Route 2 — East");
}

#[test]
fn world_has_signs_and_npcs() {
    let props = world_props();
    assert!(props.len() >= 8);
    assert!(props.iter().filter(|p| p.kind == PropKind::Sign).count() >= 4);
    assert!(props.iter().filter(|p| p.kind == PropKind::Npc).count() >= 4);
    let welcome = interact_prop_at(7, 13, 0).expect("welcome sign");
    assert_eq!(welcome.title, "Welcome Sign");
    assert!(!welcome.lines.is_empty());
}

#[test]
fn walkability_new_tiles() {
    assert!(Tile::Sand.walkable());
    assert!(Tile::Sign.walkable());
    assert!(!Tile::Fence.walkable());
    assert!(!Tile::Water.walkable());
}

#[test]
fn interact_prop_facing_only() {
    assert!(interact_prop_at(7, 13, 0).is_some());
    assert!(interact_prop_at(7, 14, 3).is_some()); // face north toward sign
    assert!(interact_prop_at(7, 14, 0).is_none()); // face south, not on prop
    assert!(interact_prop_at(6, 13, 0).is_none()); // orthogonal without facing
}

#[test]
fn npc_blocks_cells() {
    assert!(is_npc_at(4, 16)); // Nurse Joy
    assert!(is_npc_at(26, 7)); // Hiker
}

#[test]
fn dex_progress_unique_valid() {
    let mut d = DexProgress::default();
    d.mark_seen(25);
    d.mark_seen(25);
    d.mark_caught(25);
    d.mark_seen(0);
    d.seen.push(999);
    d.normalize();
    assert_eq!(d.seen_count(), 1);
    assert_eq!(d.caught_count(), 1);
    assert!(d.has_seen(25));
    assert_eq!(DEX_SIZE, 151);
}

#[test]
fn follower_uses_first_conscious() {
    let mut s = SaveGame::default();
    let mut a = PokemonInstance::from_species_id(1, 5).unwrap();
    a.current_hp = 0;
    s.party.push(a);
    s.party
        .push(PokemonInstance::from_species_id(4, 5).unwrap());
    assert_eq!(s.follower_species_id(), Some(4));
}

#[test]
fn full_party_catch_documents_dex_still_counts() {
    let mut s = SaveGame::default();
    for id in [1u16, 4, 7, 25, 10, 16] {
        s.party
            .push(PokemonInstance::from_species_id(id, 5).unwrap());
    }
    assert_eq!(s.party.len(), 6);
    s.dex.mark_caught(52);
    s.pokemon_caught += 1;
    assert_eq!(s.party.len(), 6);
    assert!(s.dex.has_caught(52));
}
