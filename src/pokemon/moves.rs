//! Move definitions.

use serde::{Deserialize, Serialize};

use super::types::ElementType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MoveCategory {
    #[default]
    Physical,
    Special,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoveData {
    pub id: u16,
    pub name: String,
    pub move_type: ElementType,
    pub category: MoveCategory,
    /// Base power; 0 for status moves.
    pub power: u8,
    /// Accuracy percent 1–100; 0 = always hits.
    pub accuracy: u8,
    pub pp: u8,
    pub priority: i8,
    pub description: String,
}

impl MoveData {
    pub fn display_name(&self) -> &str {
        &self.name
    }
}

/// Runtime instance of a move with remaining PP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearnedMove {
    pub data: MoveData,
    pub current_pp: u8,
}

impl LearnedMove {
    pub fn new(data: MoveData) -> Self {
        let pp = data.pp;
        Self {
            data,
            current_pp: pp,
        }
    }

    pub fn can_use(&self) -> bool {
        self.current_pp > 0
    }

    pub fn spend_pp(&mut self) {
        self.current_pp = self.current_pp.saturating_sub(1);
    }
}

/// Built-in starter move catalogue (subset; full catalogue lives in JSON data).
pub fn builtin_moves() -> Vec<MoveData> {
    use ElementType::*;
    use MoveCategory::*;
    let m = |id: u16,
             name: &str,
             t: ElementType,
             cat: MoveCategory,
             power: u8,
             acc: u8,
             pp: u8,
             desc: &str| MoveData {
        id,
        name: name.into(),
        move_type: t,
        category: cat,
        power,
        accuracy: acc,
        pp,
        priority: 0,
        description: desc.into(),
    };
    vec![
        m(
            1,
            "Tackle",
            Normal,
            Physical,
            40,
            100,
            35,
            "A physical attack in which the user charges and slams into the target.",
        ),
        m(
            2,
            "Scratch",
            Normal,
            Physical,
            40,
            100,
            35,
            "Hard, pointed, sharp claws rake the target.",
        ),
        m(
            3,
            "Growl",
            Normal,
            Status,
            0,
            100,
            40,
            "The user growls cutely, lowering the target's Attack.",
        ),
        m(
            4,
            "Ember",
            Fire,
            Special,
            40,
            100,
            25,
            "The target is attacked with small flames.",
        ),
        m(
            5,
            "Water Gun",
            Water,
            Special,
            40,
            100,
            25,
            "The target is blasted with a forceful shot of water.",
        ),
        m(
            6,
            "Vine Whip",
            Grass,
            Physical,
            45,
            100,
            25,
            "The target is struck with slender, whiplike vines.",
        ),
        m(
            7,
            "Thunder Shock",
            Electric,
            Special,
            40,
            100,
            30,
            "A jolt of electricity crashes down on the target.",
        ),
        m(
            8,
            "Quick Attack",
            Normal,
            Physical,
            40,
            100,
            30,
            "The user lunges at the target at a speed that makes it almost invisible.",
        ),
        m(
            9,
            "Thunderbolt",
            Electric,
            Special,
            90,
            100,
            15,
            "A strong electric blast crashes down on the target.",
        ),
        m(
            10,
            "Flamethrower",
            Fire,
            Special,
            90,
            100,
            15,
            "The target is scorched with an intense blast of fire.",
        ),
        m(
            11,
            "Surf",
            Water,
            Special,
            90,
            100,
            15,
            "The user attacks everything around it by swamping its surroundings with a giant wave.",
        ),
        m(
            12,
            "Solar Beam",
            Grass,
            Special,
            120,
            100,
            10,
            "A two-turn attack that blasts the target with a huge volume of light.",
        ),
        m(
            13,
            "Ice Beam",
            Ice,
            Special,
            90,
            100,
            10,
            "The target is struck with an icy-cold beam of energy.",
        ),
        m(
            14,
            "Psychic",
            Psychic,
            Special,
            90,
            100,
            10,
            "The target is hit by a strong telekinetic force.",
        ),
        m(
            15,
            "Earthquake",
            Ground,
            Physical,
            100,
            100,
            10,
            "The user sets off an earthquake that strikes every Pokémon around it.",
        ),
        m(
            16,
            "Rock Slide",
            Rock,
            Physical,
            75,
            90,
            10,
            "Large boulders are hurled at the opposing Pokémon.",
        ),
        m(
            17,
            "Shadow Ball",
            Ghost,
            Special,
            80,
            100,
            15,
            "The user hurls a shadowy blob at the target.",
        ),
        m(
            18,
            "Dragon Claw",
            Dragon,
            Physical,
            80,
            100,
            15,
            "The user slashes the target with huge sharp claws.",
        ),
        m(
            19,
            "Iron Tail",
            Steel,
            Physical,
            100,
            75,
            15,
            "The target is slammed with a steel-hard tail.",
        ),
        m(
            20,
            "Crunch",
            Dark,
            Physical,
            80,
            100,
            15,
            "The user crunches up the target with sharp fangs.",
        ),
        m(
            21,
            "Aerial Ace",
            Flying,
            Physical,
            60,
            0,
            20,
            "The user confounds the target with speed, then slashes. Never misses.",
        ),
        m(
            22,
            "Bug Buzz",
            Bug,
            Special,
            90,
            100,
            10,
            "The user generates a damaging sound wave by vibration.",
        ),
        m(
            23,
            "Sludge Bomb",
            Poison,
            Special,
            90,
            100,
            10,
            "Unsanitary sludge is hurled at the target.",
        ),
        m(
            24,
            "Dazzling Gleam",
            Fairy,
            Special,
            80,
            100,
            10,
            "The user damages opposing Pokémon by emitting a powerful flash.",
        ),
        m(
            25,
            "Close Combat",
            Fighting,
            Physical,
            120,
            100,
            5,
            "The user fights the target up close without guarding itself.",
        ),
        m(
            26,
            "Hyper Beam",
            Normal,
            Special,
            150,
            90,
            5,
            "The target is attacked with a powerful beam.",
        ),
        m(
            27,
            "Fire Blast",
            Fire,
            Special,
            110,
            85,
            5,
            "The target is attacked with an intense blast of all-consuming fire.",
        ),
        m(
            28,
            "Hydro Pump",
            Water,
            Special,
            110,
            80,
            5,
            "The target is blasted by a huge volume of water.",
        ),
        m(
            29,
            "Thunder",
            Electric,
            Special,
            110,
            70,
            10,
            "A wicked thunderbolt is dropped on the target.",
        ),
        m(
            30,
            "Body Slam",
            Normal,
            Physical,
            85,
            100,
            15,
            "The user drops onto the target with its full body weight.",
        ),
    ]
}

pub fn move_by_name(name: &str) -> Option<MoveData> {
    crate::pokemon::db::move_by_name(name)
}

pub fn move_by_id(id: u16) -> Option<MoveData> {
    crate::pokemon::db::move_by_id(id)
}

/// Convert legacy string move names (from old team JSON) into LearnedMove list.
pub fn learned_from_legacy_names(names: &[String]) -> Vec<LearnedMove> {
    let mut out = Vec::new();
    for n in names {
        if let Some(m) = move_by_name(n) {
            out.push(LearnedMove::new(m));
        } else {
            // Unknown custom move: treat as normal physical tackle variant
            out.push(LearnedMove::new(MoveData {
                id: 9000,
                name: n.clone(),
                move_type: ElementType::Normal,
                category: MoveCategory::Physical,
                power: 50,
                accuracy: 100,
                pp: 20,
                priority: 0,
                description: "A custom move.".into(),
            }));
        }
    }
    if out.is_empty() {
        if let Some(t) = move_by_name("Tackle") {
            out.push(LearnedMove::new(t));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tackle_exists() {
        let t = move_by_name("Tackle").unwrap();
        assert_eq!(t.power, 40);
        assert_eq!(t.move_type, ElementType::Normal);
    }

    #[test]
    fn learned_move_spends_pp() {
        let t = move_by_name("Tackle").unwrap();
        let mut lm = LearnedMove::new(t);
        assert!(lm.can_use());
        lm.spend_pp();
        assert_eq!(lm.current_pp, lm.data.pp - 1);
    }

    #[test]
    fn legacy_names_convert() {
        let names = vec!["Thunderbolt".into(), "CustomMoveX".into()];
        let moves = learned_from_legacy_names(&names);
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].data.name, "Thunderbolt");
        assert_eq!(moves[1].data.name, "CustomMoveX");
    }
}
