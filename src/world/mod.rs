//! Simplified overworld: one map, wild grass, evolutions, NPCs/signs.

/// Tile kinds for the overworld grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tile {
    Grass = 0,
    TallGrass = 1,
    Path = 2,
    Water = 3,
    Tree = 4,
    Building = 5,
    Floor = 6,
    Door = 7,
    Counter = 8,
    Flower = 9,
    /// Blue mart storefront (interact with E / step on door).
    Mart = 10,
    MartDoor = 11,
    /// Light sandy shore / beach edge near water.
    Sand = 12,
    /// Wooden fence (blocks walking).
    Fence = 13,
    /// Signpost tile (walkable; interact with E).
    Sign = 14,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(
            self,
            Self::Water | Self::Tree | Self::Building | Self::Counter | Self::Mart | Self::Fence
        )
    }

    pub fn is_tall_grass(self) -> bool {
        matches!(self, Self::TallGrass)
    }

    /// Standing on door or center floor heals when interacting.
    pub fn is_center_area(self) -> bool {
        matches!(self, Self::Floor | Self::Door)
    }

    pub fn is_mart_area(self) -> bool {
        matches!(self, Self::MartDoor)
    }
}

/// Map width / height in tiles.
pub const MAP_W: usize = 32;
pub const MAP_H: usize = 24;
/// Larger tiles = easier to read at 1024×680.
pub const TILE_PX: f32 = 40.0;

/// Sign vs person on the overworld.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    Sign,
    Npc,
}

impl PropKind {
    pub fn is_npc(self) -> bool {
        matches!(self, Self::Npc)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sign => "Sign",
            Self::Npc => "NPC",
        }
    }
}

/// Static overworld prop: sign or NPC (interact with E on tile or strictly facing tile).
#[derive(Debug, Clone, Copy)]
pub struct WorldProp {
    pub tx: i32,
    pub ty: i32,
    pub kind: PropKind,
    pub title: &'static str,
    pub lines: &'static [&'static str],
}

/// True when an NPC occupies this grid cell (blocks walking; talk by facing them).
pub fn is_npc_at(tx: i32, ty: i32) -> bool {
    world_props()
        .iter()
        .any(|p| p.kind.is_npc() && p.tx == tx && p.ty == ty)
}

/// All interactable props on the default map.
pub fn world_props() -> &'static [WorldProp] {
    &[
        WorldProp {
            tx: 7,
            ty: 13,
            kind: PropKind::Sign,
            title: "Welcome Sign",
            lines: &[
                "Welcome to Pallet Town!",
                "Red door/floor = Center (E heals).",
                "Blue mart door = shop (E opens).",
                "Stand on this sign or face it + E.",
            ],
        },
        WorldProp {
            tx: 11,
            ty: 8,
            kind: PropKind::Sign,
            title: "Route 1",
            lines: &[
                "Route 1 — North Grass",
                "Wild: Caterpie, Weedle, Pidgey,",
                "Rattata, Spearow  ·  Lv 3–7",
                "Good training for new trainers!",
            ],
        },
        WorldProp {
            tx: 21,
            ty: 10,
            kind: PropKind::Sign,
            title: "Route 2",
            lines: &[
                "Route 2 — East Wilds",
                "Stronger Pokémon appear here.",
                "Wild: Pikachu, Sandshrew, Nidoran,",
                "Clefairy, Vulpix & more  ·  Lv 6–12",
            ],
        },
        WorldProp {
            tx: 15,
            ty: 17,
            kind: PropKind::Sign,
            title: "Route 3",
            lines: &[
                "Route 3 — Deep Grass",
                "Tough area! Bring potions.",
                "Wild: Psyduck, Mankey, Growlithe,",
                "Abra, Machop & more  ·  Lv 8–15",
            ],
        },
        WorldProp {
            tx: 22,
            ty: 18,
            kind: PropKind::Sign,
            title: "Pond Notice",
            lines: &[
                "Scenic pond — no surfing yet!",
                "Water blocks travel; go around.",
                "Sandy shores make nice landmarks.",
            ],
        },
        WorldProp {
            tx: 4,
            ty: 16,
            kind: PropKind::Npc,
            title: "Nurse Joy",
            lines: &[
                "Welcome to the Pokémon Center!",
                "Step on our door for a full heal,",
                "or press E while on/facing the door.",
                "We'll top up a few supplies too!",
            ],
        },
        WorldProp {
            tx: 12,
            ty: 19,
            kind: PropKind::Npc,
            title: "Shop Clerk",
            lines: &[
                "Need supplies? Try the Mart!",
                "Balls $100 · Potions $80",
                "Ball ×3 deal saves you $30.",
                "Stand on blue door, or face it + E.",
            ],
        },
        WorldProp {
            tx: 8,
            ty: 6,
            kind: PropKind::Npc,
            title: "Veteran Trainer",
            lines: &[
                "Type matchups win battles!",
                "Water beats Fire; Electric fails",
                "vs Ground. SE moves show green",
                "in the Fight menu. Catch low-HP foes!",
            ],
        },
        WorldProp {
            tx: 18,
            ty: 11,
            kind: PropKind::Npc,
            title: "Bug Catcher",
            lines: &[
                "I love the tall grass!",
                "Swaying blades mean encounters.",
                "If your whole party faints you'll",
                "wake at the Center — don't worry.",
            ],
        },
        WorldProp {
            tx: 26,
            ty: 7,
            kind: PropKind::Npc,
            title: "Hiker",
            lines: &[
                "East route has rarer finds.",
                "Pikachu hides in those weeds!",
                "Build your party, then push east.",
                "Press P for party · Esc to save.",
            ],
        },
    ]
}

fn prop_at(tx: i32, ty: i32) -> Option<&'static WorldProp> {
    world_props().iter().find(|p| p.tx == tx && p.ty == ty)
}

/// Facing offsets: 0 down, 1 left, 2 right, 3 up.
pub fn facing_delta(facing: u8) -> (i32, i32) {
    match facing {
        1 => (-1, 0),
        2 => (1, 0),
        3 => (0, -1),
        _ => (0, 1),
    }
}

/// Prop interact: **only** standing on a prop tile, or strictly facing one neighbor cell.
/// No "any adjacent" fallback — avoids stealing Center/Mart E and list-order ambiguity.
pub fn interact_prop_at(px: i32, py: i32, facing: u8) -> Option<&'static WorldProp> {
    if let Some(p) = prop_at(px, py) {
        return Some(p);
    }
    let (fx, fy) = facing_delta(facing);
    prop_at(px + fx, py + fy)
}

/// Build the default adventure map (town + routes + grass + center).
pub fn build_map() -> Vec<Vec<Tile>> {
    let mut m = vec![vec![Tile::Grass; MAP_W]; MAP_H];

    for x in 0..MAP_W {
        m[0][x] = Tile::Tree;
        m[MAP_H - 1][x] = Tile::Tree;
    }
    for y in 0..MAP_H {
        m[y][0] = Tile::Tree;
        m[y][MAP_W - 1] = Tile::Tree;
    }

    // Town plaza
    for y in 14..21 {
        for x in 3..14 {
            m[y][x] = Tile::Path;
        }
    }

    // Pokemon Center building
    for y in 15..19 {
        for x in 5..10 {
            m[y][x] = Tile::Building;
        }
    }
    m[18][7] = Tile::Door;
    for y in 19..21 {
        for x in 5..10 {
            m[y][x] = Tile::Floor;
        }
    }

    for x in 11..13 {
        m[16][x] = Tile::Flower;
        m[17][x] = Tile::Flower;
    }

    // Mart (blue shop, east of center)
    for y in 15..18 {
        for x in 11..14 {
            m[y][x] = Tile::Mart;
        }
    }
    m[17][12] = Tile::MartDoor;

    // Decorative town fences
    for x in 3..5 {
        m[14][x] = Tile::Fence;
    }
    for x in 12..14 {
        m[14][x] = Tile::Fence;
    }

    // Paths
    for y in 5..15 {
        for x in 7..10 {
            m[y][x] = Tile::Path;
        }
    }
    for y in 9..12 {
        for x in 10..28 {
            m[y][x] = Tile::Path;
        }
    }

    // Tall grass
    for y in 3..9 {
        for x in 12..20 {
            if m[y][x] == Tile::Grass {
                m[y][x] = Tile::TallGrass;
            }
        }
    }
    for y in 12..18 {
        for x in 16..26 {
            if m[y][x] == Tile::Grass {
                m[y][x] = Tile::TallGrass;
            }
        }
    }
    for y in 4..10 {
        for x in 22..29 {
            if m[y][x] == Tile::Grass {
                m[y][x] = Tile::TallGrass;
            }
        }
    }

    // Pond + sandy shore
    for y in 19..22 {
        for x in 20..26 {
            m[y][x] = Tile::Water;
        }
    }
    for x in 20..26 {
        if matches!(m[18][x], Tile::Grass | Tile::TallGrass) {
            m[18][x] = Tile::Sand;
        }
        if matches!(m[22][x], Tile::Grass | Tile::TallGrass) {
            m[22][x] = Tile::Sand;
        }
    }
    for y in 19..22 {
        if matches!(m[y][19], Tile::Grass | Tile::TallGrass) {
            m[y][19] = Tile::Sand;
        }
        if matches!(m[y][26], Tile::Grass | Tile::TallGrass) {
            m[y][26] = Tile::Sand;
        }
    }

    // Extra flower patches near town
    for &(y, x) in &[(20, 4), (20, 13), (15, 4)] {
        if y < MAP_H && x < MAP_W && matches!(m[y][x], Tile::Grass | Tile::Path) {
            m[y][x] = Tile::Flower;
        }
    }

    // Scatter trees for biome feel
    for &(y, x) in &[
        (4, 4),
        (5, 5),
        (6, 3),
        (12, 4),
        (13, 13),
        (8, 28),
        (20, 15),
        (3, 10),
        (7, 21),
        (14, 28),
        (21, 28),
        (11, 2),
        (16, 2),
        (2, 16),
        (2, 24),
    ] {
        if y < MAP_H && x < MAP_W && m[y][x] == Tile::Grass {
            m[y][x] = Tile::Tree;
        }
    }

    // Sign tiles (walkable markers under sign props)
    for prop in world_props() {
        if prop.kind == PropKind::Sign {
            let x = prop.tx as usize;
            let y = prop.ty as usize;
            if y < MAP_H
                && x < MAP_W
                && m[y][x].walkable()
                && m[y][x] != Tile::Door
                && m[y][x] != Tile::MartDoor
            {
                m[y][x] = Tile::Sign;
            }
        }
    }

    m
}

/// Clamp / snap player grid to a walkable in-bounds tile (post-load safety).
pub fn normalize_spawn(tx: i32, ty: i32, map: &[Vec<Tile>]) -> (i32, i32) {
    let mut x = tx.clamp(1, MAP_W as i32 - 2);
    let mut y = ty.clamp(1, MAP_H as i32 - 2);
    let ok = |x: i32, y: i32| {
        let t = map[y as usize][x as usize];
        t.walkable() && !is_npc_at(x, y)
    };
    if ok(x, y) {
        return (x, y);
    }
    // Spiral search for nearest walkable
    for r in 1..16 {
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 1 || ny < 1 || nx >= MAP_W as i32 - 1 || ny >= MAP_H as i32 - 1 {
                    continue;
                }
                if ok(nx, ny) {
                    return (nx, ny);
                }
            }
        }
    }
    // Hard fallback: default starter path
    x = 8;
    y = 10;
    (x, y)
}

/// Wild encounter pools by map region (ids are Gen1; filtered at battle start if missing in DB).
pub fn wild_pool_for_tile(tx: i32, ty: i32) -> (u8, u8, &'static [u16]) {
    if ty < 9 && (12..20).contains(&tx) {
        return (3, 7, &[10, 13, 16, 19, 21]);
    }
    if tx >= 22 {
        return (6, 12, &[25, 27, 29, 32, 35, 37, 39, 41, 43, 46, 48, 52]);
    }
    if ty >= 12 && tx >= 16 {
        return (
            8,
            15,
            &[
                54, 56, 58, 60, 63, 66, 69, 72, 74, 77, 79, 81, 84, 86, 88, 92, 96,
            ],
        );
    }
    (4, 9, &[10, 13, 16, 19, 21, 23, 25, 27])
}

/// Short area label for HUD (helps players orient).
pub fn area_name_for_tile(tx: i32, ty: i32) -> &'static str {
    if ty >= 14 && (3..14).contains(&tx) {
        return "Pallet Town";
    }
    // Pond before east-route bucket so shoreline tiles label correctly.
    if ty >= 18 && (19..=27).contains(&tx) {
        return "South Pond";
    }
    if ty < 9 && (12..20).contains(&tx) {
        return "Route 1 — North";
    }
    if tx >= 22 {
        return "Route 2 — East";
    }
    if ty >= 12 && tx >= 16 {
        return "Route 3 — Deep Grass";
    }
    if (9..12).contains(&ty) && tx >= 10 {
        return "Crossroads";
    }
    "The Wilds"
}

/// Simple evolution table: (from_id, min_level, to_id).
pub fn evolution_for(species_id: u16, level: u8) -> Option<u16> {
    let table: &[(u16, u8, u16)] = &[
        (1, 16, 2),
        (2, 32, 3),
        (4, 16, 5),
        (5, 36, 6),
        (7, 16, 8),
        (8, 36, 9),
        (10, 7, 11),
        (11, 10, 12),
        (13, 7, 14),
        (14, 10, 15),
        (16, 18, 17),
        (17, 36, 18),
        (19, 20, 20),
        (21, 20, 22),
        (23, 22, 24),
        (27, 22, 28),
        (29, 16, 30),
        (32, 16, 33),
        (41, 22, 42),
        (43, 21, 44),
        (46, 24, 47),
        (48, 31, 49),
        (54, 33, 55),
        (56, 28, 57),
        (60, 25, 61),
        (63, 16, 64),
        (66, 28, 67),
        (69, 21, 70),
        (72, 30, 73),
        (74, 25, 75),
        (77, 40, 78),
        (79, 37, 80),
        (81, 30, 82),
        (84, 31, 85),
        (86, 34, 87),
        (88, 38, 89),
        (92, 25, 93),
        (96, 26, 97),
        (98, 28, 99),
        (100, 30, 101),
        (104, 28, 105),
        (109, 35, 110),
        (111, 42, 112),
        (116, 32, 117),
        (118, 33, 119),
        (129, 20, 130),
        (138, 40, 139),
        (140, 40, 141),
        (147, 30, 148),
        (148, 55, 149),
    ];
    for &(from, min_lv, to) in table {
        if from == species_id && to > 0 && level >= min_lv {
            return Some(to);
        }
    }
    None
}

pub fn starter_species() -> [(u16, &'static str); 3] {
    [(1, "Bulbasaur"), (4, "Charmander"), (7, "Squirtle")]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weather {
    #[default]
    Clear,
}

impl Weather {
    pub fn display_name(self) -> &'static str {
        "Clear"
    }
    pub fn move_multiplier(self, _move_type: crate::pokemon::types::ElementType) -> f64 {
        1.0
    }
    pub fn residual_damage_type(self) -> Option<crate::pokemon::types::ElementType> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn props_cover_signs_and_npcs() {
        let props = world_props();
        assert!(props.len() >= 8);
        assert!(props.iter().any(|p| p.kind == PropKind::Sign));
        assert!(props.iter().any(|p| p.kind == PropKind::Npc));
    }

    #[test]
    fn interact_on_tile_and_facing_only() {
        let on = interact_prop_at(7, 13, 0).expect("on sign");
        assert_eq!(on.title, "Welcome Sign");
        // Standing south of sign facing up (3) sees sign
        let face = interact_prop_at(7, 14, 3).expect("facing sign");
        assert_eq!(face.title, "Welcome Sign");
        // Standing south facing down — no prop
        assert!(interact_prop_at(7, 14, 0).is_none());
        // Adjacent but not facing (orthogonal without facing) — no prop
        assert!(interact_prop_at(6, 13, 0).is_none());
        assert!(interact_prop_at(8, 13, 0).is_none());
    }

    #[test]
    fn walkability_sand_fence_sign() {
        assert!(Tile::Sand.walkable());
        assert!(Tile::Sign.walkable());
        assert!(!Tile::Fence.walkable());
        assert!(!Tile::Water.walkable());
        assert!(!Tile::Mart.walkable());
    }

    #[test]
    fn npc_blocks_and_map_props_consistent() {
        let m = build_map();
        for prop in world_props() {
            let x = prop.tx as usize;
            let y = prop.ty as usize;
            assert!(y < MAP_H && x < MAP_W);
            if prop.kind == PropKind::Sign {
                assert_eq!(m[y][x], Tile::Sign);
            }
            if prop.kind == PropKind::Npc {
                assert!(is_npc_at(prop.tx, prop.ty));
            }
        }
        // Nurse Joy blocks (4,16)
        assert!(is_npc_at(4, 16));
        assert!(m[16][4].walkable()); // tile may be path/floor, NPC still blocks via is_npc_at
    }

    #[test]
    fn normalize_spawn_clamps() {
        let m = build_map();
        let (x, y) = normalize_spawn(-5, 999, &m);
        assert!(x >= 1 && y >= 1);
        assert!(x < MAP_W as i32 && y < MAP_H as i32);
        assert!(m[y as usize][x as usize].walkable());
        assert!(!is_npc_at(x, y));
    }
}
