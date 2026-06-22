//! Species definitions (PokeAPI-style metadata).

use serde::{Deserialize, Serialize};

use super::stats::BaseStats;
use super::types::ElementType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Species {
    pub id: u16,
    pub name: String,
    pub types: Vec<ElementType>,
    pub base_stats: BaseStats,
    pub base_experience: u32,
    pub height_dm: u16,
    pub weight_hg: u16,
    pub description: String,
    /// Default move ids for wild / starter instances.
    pub default_moves: Vec<u16>,
    pub capture_rate: u8,
}

impl Species {
    pub fn primary_type(&self) -> ElementType {
        self.types.first().copied().unwrap_or(ElementType::Normal)
    }

    pub fn type_label(&self) -> String {
        self.types
            .iter()
            .map(|t| t.display_name())
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn sprite_key(&self) -> String {
        format!("{:03}", self.id)
    }
}

/// Build the embedded Gen 1 (+ select later) species catalogue.
pub fn builtin_species() -> Vec<Species> {
    use ElementType::*;
    let s = |id: u16,
             name: &str,
             types: Vec<ElementType>,
             hp: u16,
             atk: u16,
             def: u16,
             spa: u16,
             spd: u16,
             spe: u16,
             base_exp: u32,
             moves: &[u16],
             desc: &str| Species {
        id,
        name: name.into(),
        types,
        base_stats: BaseStats::new(hp, atk, def, spa, spd, spe),
        base_experience: base_exp,
        height_dm: 7,
        weight_hg: 69,
        description: desc.into(),
        default_moves: moves.to_vec(),
        capture_rate: 45,
    };

    vec![
        s(1, "Bulbasaur", vec![Grass, Poison], 45, 49, 49, 65, 65, 45, 64, &[1, 6], "A strange seed was planted on its back at birth. The plant sprouts and grows with this Pokémon."),
        s(2, "Ivysaur", vec![Grass, Poison], 60, 62, 63, 80, 80, 60, 142, &[1, 6, 12], "When the bulb on its back grows large, it appears to lose the ability to stand on its hind legs."),
        s(3, "Venusaur", vec![Grass, Poison], 80, 82, 83, 100, 100, 80, 236, &[6, 12, 23], "Its plant blooms when it is absorbing solar energy. It stays on the move to seek sunlight."),
        s(4, "Charmander", vec![Fire], 39, 52, 43, 60, 50, 65, 62, &[1, 4], "Obviously prefers hot places. When it rains, steam is said to spout from the tip of its tail."),
        s(5, "Charmeleon", vec![Fire], 58, 64, 58, 80, 65, 80, 142, &[1, 4, 10], "When it swings its burning tail, it elevates the temperature to unbearably high levels."),
        s(6, "Charizard", vec![Fire, Flying], 78, 84, 78, 109, 85, 100, 240, &[4, 10, 18, 21], "Spits fire that is hot enough to melt boulders. Known to cause forest fires unintentionally."),
        s(7, "Squirtle", vec![Water], 44, 48, 65, 50, 64, 43, 63, &[1, 5], "After birth, its back swells and hardens into a shell. Powerfully sprays foam from its mouth."),
        s(8, "Wartortle", vec![Water], 59, 63, 80, 65, 80, 58, 142, &[1, 5, 11], "Often hides in water to stalk unwary prey. For swimming fast, it moves its ears to maintain balance."),
        s(9, "Blastoise", vec![Water], 79, 83, 100, 85, 105, 78, 239, &[5, 11, 28], "A brutal Pokémon with pressurized water jets on its shell. They are used for high speed tackles."),
        s(10, "Caterpie", vec![Bug], 45, 30, 35, 20, 20, 45, 39, &[1], "Its short feet are tipped with suction pads that enable it to tirelessly climb slopes and walls."),
        s(16, "Pidgey", vec![Normal, Flying], 40, 45, 40, 35, 35, 56, 50, &[1, 21], "A common sight in forests and woods. It flaps its wings at ground level to kick up blinding sand."),
        s(19, "Rattata", vec![Normal], 30, 56, 35, 25, 35, 72, 51, &[1, 8], "Bites anything when it attacks. Small and very quick, it is a common sight in many places."),
        s(25, "Pikachu", vec![Electric], 35, 55, 40, 50, 50, 90, 112, &[7, 8, 9, 19], "When several of these Pokémon gather, their electricity could build and cause lightning storms."),
        s(26, "Raichu", vec![Electric], 60, 90, 55, 90, 80, 110, 218, &[7, 9, 29], "Its long tail serves as a ground to protect itself from its own high voltage power."),
        s(39, "Jigglypuff", vec![Normal, Fairy], 115, 45, 20, 45, 25, 20, 95, &[1, 24], "When its huge eyes light up, it sings a mysteriously soothing melody that lulls its enemies to sleep."),
        s(52, "Meowth", vec![Normal], 40, 45, 35, 40, 40, 90, 58, &[2, 8], "Adores circular objects. Wanders the streets on a nightly basis to look for dropped loose change."),
        s(58, "Growlithe", vec![Fire], 55, 70, 45, 70, 50, 60, 70, &[1, 4, 10], "Very protective of its territory. It will bark and bite to repel intruders from its space."),
        s(63, "Abra", vec![Psychic], 25, 20, 15, 105, 55, 90, 62, &[14], "Using its ability to read minds, it will identify impending danger and teleport to safety."),
        s(66, "Machop", vec![Fighting], 70, 80, 50, 35, 35, 35, 61, &[1, 25], "Loves to build its muscles. It trains in all styles of martial arts to become even stronger."),
        s(74, "Geodude", vec![Rock, Ground], 40, 80, 100, 30, 30, 20, 60, &[1, 16], "Found in fields and mountains. Mistaking them for boulders, people often step or trip on them."),
        s(92, "Gastly", vec![Ghost, Poison], 30, 35, 30, 100, 35, 80, 62, &[17, 23], "Almost invisible, this gaseous Pokémon cloaks the target and puts it to sleep without notice."),
        s(94, "Gengar", vec![Ghost, Poison], 60, 65, 60, 130, 75, 110, 225, &[17, 14, 23], "Under a full moon, this Pokémon likes to mimic the shadows of people and laugh at their fright."),
        s(95, "Onix", vec![Rock, Ground], 35, 45, 160, 30, 45, 70, 77, &[1, 15, 16], "As it grows, the stone portions of its body harden to become similar to a diamond, but colored black."),
        s(129, "Magikarp", vec![Water], 20, 10, 55, 15, 20, 80, 40, &[1], "In the distant past, it was somewhat stronger than the horribly weak descendants that exist today."),
        s(130, "Gyarados", vec![Water, Flying], 95, 125, 79, 60, 100, 81, 189, &[5, 11, 20, 21], "Rarely seen in the wild. Huge and vicious, it is capable of destroying entire cities in a rage."),
        s(131, "Lapras", vec![Water, Ice], 130, 85, 80, 85, 95, 60, 187, &[5, 11, 13], "A Pokémon that has been overhunted almost to extinction. It can ferry people across the water."),
        s(133, "Eevee", vec![Normal], 55, 55, 50, 45, 65, 55, 65, &[1, 8], "Its genetic code is irregular. It may mutate if it is exposed to radiation from element stones."),
        s(143, "Snorlax", vec![Normal], 160, 110, 65, 65, 110, 30, 189, &[1, 30, 26], "Very lazy. Just eats and sleeps. As its rotund bulk builds, it becomes steadily more slothful."),
        s(149, "Dragonite", vec![Dragon, Flying], 91, 134, 95, 100, 100, 80, 270, &[18, 21, 10, 11], "An extremely rarely seen marine Pokémon. Its intelligence is said to match that of humans."),
        s(150, "Mewtwo", vec![Psychic], 106, 110, 90, 154, 90, 130, 306, &[14, 9, 13, 26], "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments."),
        s(151, "Mew", vec![Psychic], 100, 100, 100, 100, 100, 100, 270, &[14, 24, 9, 10], "So rare that it is still said to be a mirage by many experts. Only a few people have seen it worldwide."),
        // Custom easter egg preserved from original game
        s(999, "Siddharth", vec![Normal], 100, 120, 80, 90, 80, 95, 200, &[1, 25, 26, 20], "A legendary trainer-turned-Pokémon. Its moves include Sarcastic Comments and DeadEye."),
    ]
}

pub fn species_by_id(id: u16) -> Option<Species> {
    crate::pokemon::db::species_by_id(id)
}

pub fn species_by_name(name: &str) -> Option<Species> {
    crate::pokemon::db::species_by_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pikachu_is_electric() {
        let p = species_by_name("Pikachu").unwrap();
        assert_eq!(p.id, 25);
        assert_eq!(p.primary_type(), ElementType::Electric);
    }

    #[test]
    fn charizard_dual_type() {
        let c = species_by_id(6).unwrap();
        assert_eq!(c.types.len(), 2);
        assert_eq!(c.type_label(), "Fire/Flying");
    }

    #[test]
    fn catalogue_not_empty() {
        assert!(builtin_species().len() >= 30);
    }
}
