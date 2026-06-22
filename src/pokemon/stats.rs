//! Base stats and level scaling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BaseStats {
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
}

impl BaseStats {
    pub const fn new(
        hp: u16,
        attack: u16,
        defense: u16,
        sp_attack: u16,
        sp_defense: u16,
        speed: u16,
    ) -> Self {
        Self {
            hp,
            attack,
            defense,
            sp_attack,
            sp_defense,
            speed,
        }
    }

    /// Simple level formula approximating main-series stat calculation.
    pub fn stat_at_level(&self, base: u16, level: u8) -> i64 {
        let level = level.max(1) as u32;
        let value = ((2 * base as u32) * level / 100) + 5;
        value as i64
    }

    pub fn max_hp_at_level(&self, level: u8) -> i64 {
        let level = level.max(1) as u32;
        let value = ((2 * self.hp as u32) * level / 100) + level + 10;
        value as i64
    }

    pub fn attack_at_level(&self, level: u8) -> i64 {
        self.stat_at_level(self.attack, level)
    }

    pub fn defense_at_level(&self, level: u8) -> i64 {
        self.stat_at_level(self.defense, level)
    }

    pub fn sp_attack_at_level(&self, level: u8) -> i64 {
        self.stat_at_level(self.sp_attack, level)
    }

    pub fn sp_defense_at_level(&self, level: u8) -> i64 {
        self.stat_at_level(self.sp_defense, level)
    }

    pub fn speed_at_level(&self, level: u8) -> i64 {
        self.stat_at_level(self.speed, level)
    }
}

/// Experience points needed for next level (medium-fast curve approximation).
pub fn xp_for_level(level: u8) -> u32 {
    let l = level.max(1) as u32;
    l * l * l
}

pub fn xp_gain_from_defeat(defeated_level: u8, base_yield: u32) -> u32 {
    let l = defeated_level.max(1) as u32;
    (base_yield * l / 7).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_scales_with_level() {
        let s = BaseStats::new(45, 49, 49, 65, 65, 45);
        assert!(s.max_hp_at_level(50) > s.max_hp_at_level(5));
        assert!(s.max_hp_at_level(1) >= 10);
    }

    #[test]
    fn xp_curve_increases() {
        assert!(xp_for_level(10) > xp_for_level(5));
        assert!(xp_gain_from_defeat(20, 64) > 0);
    }
}
