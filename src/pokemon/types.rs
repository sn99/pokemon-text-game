//! Elemental types and effectiveness multipliers (Gen 1–6 style chart).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Pokemon elemental type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ElementType {
    #[default]
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl ElementType {
    pub const ALL: [Self; 18] = [
        Self::Normal,
        Self::Fire,
        Self::Water,
        Self::Electric,
        Self::Grass,
        Self::Ice,
        Self::Fighting,
        Self::Poison,
        Self::Ground,
        Self::Flying,
        Self::Psychic,
        Self::Bug,
        Self::Rock,
        Self::Ghost,
        Self::Dragon,
        Self::Dark,
        Self::Steel,
        Self::Fairy,
    ];

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        match s.as_str() {
            "normal" => Some(Self::Normal),
            "fire" => Some(Self::Fire),
            "water" => Some(Self::Water),
            "electric" => Some(Self::Electric),
            "grass" => Some(Self::Grass),
            "ice" => Some(Self::Ice),
            "fighting" => Some(Self::Fighting),
            "poison" => Some(Self::Poison),
            "ground" => Some(Self::Ground),
            "flying" => Some(Self::Flying),
            "psychic" => Some(Self::Psychic),
            "bug" => Some(Self::Bug),
            "rock" => Some(Self::Rock),
            "ghost" => Some(Self::Ghost),
            "dragon" => Some(Self::Dragon),
            "dark" => Some(Self::Dark),
            "steel" => Some(Self::Steel),
            "fairy" => Some(Self::Fairy),
            _ => None,
        }
    }

    /// Legacy numeric type id used by the old team file format.
    pub fn from_legacy_id(id: i64) -> Self {
        match id {
            0 => Self::Normal,
            1 => Self::Electric,
            2 => Self::Grass,
            3 => Self::Water,
            4 => Self::Fire,
            5 => Self::Fire, // Charizard-ish
            6 => Self::Psychic,
            7 => Self::Fighting,
            8 => Self::Ghost,
            9 => Self::Dragon,
            _ => Self::Normal,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Fire => "Fire",
            Self::Water => "Water",
            Self::Electric => "Electric",
            Self::Grass => "Grass",
            Self::Ice => "Ice",
            Self::Fighting => "Fighting",
            Self::Poison => "Poison",
            Self::Ground => "Ground",
            Self::Flying => "Flying",
            Self::Psychic => "Psychic",
            Self::Bug => "Bug",
            Self::Rock => "Rock",
            Self::Ghost => "Ghost",
            Self::Dragon => "Dragon",
            Self::Dark => "Dark",
            Self::Steel => "Steel",
            Self::Fairy => "Fairy",
        }
    }

    /// RGB for 2D UI type labels.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Normal => (168, 168, 120),
            Self::Fire => (240, 80, 48),
            Self::Water => (64, 144, 240),
            Self::Electric => (248, 208, 48),
            Self::Grass => (72, 192, 72),
            Self::Ice => (96, 208, 232),
            Self::Fighting => (192, 48, 40),
            Self::Poison => (160, 64, 160),
            Self::Ground => (224, 192, 104),
            Self::Flying => (168, 144, 240),
            Self::Psychic => (248, 88, 136),
            Self::Bug => (168, 184, 32),
            Self::Rock => (184, 160, 56),
            Self::Ghost => (112, 88, 152),
            Self::Dragon => (112, 56, 248),
            Self::Dark => (112, 88, 72),
            Self::Steel => (184, 184, 208),
            Self::Fairy => (240, 152, 176),
        }
    }
}

impl fmt::Display for ElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Effectiveness multiplier between attacking type and defending type(s).
pub fn type_effectiveness(attack: ElementType, defend: &[ElementType]) -> f64 {
    defend
        .iter()
        .map(|&d| single_effectiveness(attack, d))
        .product()
}

fn single_effectiveness(atk: ElementType, def: ElementType) -> f64 {
    use ElementType::*;
    match (atk, def) {
        // Normal
        (Normal, Rock | Steel) => 0.5,
        (Normal, Ghost) => 0.0,
        // Fire
        (Fire, Fire | Water | Rock | Dragon) => 0.5,
        (Fire, Grass | Ice | Bug | Steel) => 2.0,
        // Water
        (Water, Water | Grass | Dragon) => 0.5,
        (Water, Fire | Ground | Rock) => 2.0,
        // Electric
        (Electric, Electric | Grass | Dragon) => 0.5,
        (Electric, Water | Flying) => 2.0,
        (Electric, Ground) => 0.0,
        // Grass
        (Grass, Fire | Grass | Poison | Flying | Bug | Dragon | Steel) => 0.5,
        (Grass, Water | Ground | Rock) => 2.0,
        // Ice
        (Ice, Fire | Water | Ice | Steel) => 0.5,
        (Ice, Grass | Ground | Flying | Dragon) => 2.0,
        // Fighting
        (Fighting, Poison | Flying | Psychic | Bug | Fairy) => 0.5,
        (Fighting, Normal | Ice | Rock | Dark | Steel) => 2.0,
        (Fighting, Ghost) => 0.0,
        // Poison
        (Poison, Poison | Ground | Rock | Ghost) => 0.5,
        (Poison, Grass | Fairy) => 2.0,
        (Poison, Steel) => 0.0,
        // Ground
        (Ground, Grass | Bug) => 0.5,
        (Ground, Fire | Electric | Poison | Rock | Steel) => 2.0,
        (Ground, Flying) => 0.0,
        // Flying
        (Flying, Electric | Rock | Steel) => 0.5,
        (Flying, Grass | Fighting | Bug) => 2.0,
        // Psychic
        (Psychic, Psychic | Steel) => 0.5,
        (Psychic, Fighting | Poison) => 2.0,
        (Psychic, Dark) => 0.0,
        // Bug
        (Bug, Fire | Fighting | Poison | Flying | Ghost | Steel | Fairy) => 0.5,
        (Bug, Grass | Psychic | Dark) => 2.0,
        // Rock
        (Rock, Fighting | Ground | Steel) => 0.5,
        (Rock, Fire | Ice | Flying | Bug) => 2.0,
        // Ghost
        (Ghost, Dark) => 0.5,
        (Ghost, Psychic | Ghost) => 2.0,
        (Ghost, Normal) => 0.0,
        // Dragon
        (Dragon, Steel) => 0.5,
        (Dragon, Dragon) => 2.0,
        (Dragon, Fairy) => 0.0,
        // Dark
        (Dark, Fighting | Dark | Fairy) => 0.5,
        (Dark, Psychic | Ghost) => 2.0,
        // Steel
        (Steel, Fire | Water | Electric | Steel) => 0.5,
        (Steel, Ice | Rock | Fairy) => 2.0,
        // Fairy
        (Fairy, Fire | Poison | Steel) => 0.5,
        (Fairy, Fighting | Dragon | Dark) => 2.0,
        _ => 1.0,
    }
}

pub fn effectiveness_label(mult: f64) -> &'static str {
    if mult == 0.0 {
        "It has no effect..."
    } else if mult >= 2.0 {
        "It's super effective!"
    } else if mult <= 0.5 {
        "It's not very effective..."
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_beats_fire() {
        assert_eq!(
            type_effectiveness(ElementType::Water, &[ElementType::Fire]),
            2.0
        );
    }

    #[test]
    fn electric_immune_ground() {
        assert_eq!(
            type_effectiveness(ElementType::Electric, &[ElementType::Ground]),
            0.0
        );
    }

    #[test]
    fn dual_type_multiplies() {
        // Grass vs Water/Ground = 2 * 2 = 4
        let mult = type_effectiveness(
            ElementType::Grass,
            &[ElementType::Water, ElementType::Ground],
        );
        assert!((mult - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn from_str_loose_works() {
        assert_eq!(ElementType::from_str_loose("Fire"), Some(ElementType::Fire));
        assert_eq!(
            ElementType::from_str_loose("  water "),
            Some(ElementType::Water)
        );
        assert_eq!(ElementType::from_str_loose("nope"), None);
    }
}
