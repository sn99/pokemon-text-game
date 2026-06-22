//! Save / load adventure progress (simplified).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::pokemon::species::species_by_id;
use crate::pokemon::PokemonInstance;
use crate::world::{normalize_spawn, Tile, MAP_H, MAP_W};

/// Official Gen1 dex size for this game build.
pub const DEX_SIZE: u16 = 151;
/// Refuse saves larger than this (bytes) to limit OOM/hang from crafted files.
pub const MAX_SAVE_BYTES: u64 = 512 * 1024;

/// Char-safe length limit (never panics on multi-byte UTF-8; unlike `String::truncate`).
pub fn clamp_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    PokeBall,
    Potion,
}

impl ItemKind {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::PokeBall => "Poke Ball",
            Self::Potion => "Potion",
        }
    }

    pub fn catch_modifier(self) -> f64 {
        match self {
            Self::PokeBall => 1.0,
            _ => 0.0,
        }
    }

    pub fn is_ball(self) -> bool {
        matches!(self, Self::PokeBall)
    }

    pub fn heal_amount(self) -> i64 {
        match self {
            Self::Potion => 40,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryItem {
    pub kind: ItemKind,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DexProgress {
    pub seen: Vec<u16>,
    pub caught: Vec<u16>,
}

impl DexProgress {
    pub fn mark_seen(&mut self, id: u16) {
        if !Self::valid_id(id) {
            return;
        }
        if !self.seen.contains(&id) {
            self.seen.push(id);
        }
    }

    pub fn mark_caught(&mut self, id: u16) {
        if !Self::valid_id(id) {
            return;
        }
        self.mark_seen(id);
        if !self.caught.contains(&id) {
            self.caught.push(id);
        }
    }

    pub fn valid_id(id: u16) -> bool {
        (1..=DEX_SIZE).contains(&id)
    }

    /// Dedupe and drop out-of-range ids (post-load / pre-save).
    pub fn normalize(&mut self) {
        self.seen = Self::clean_ids(&self.seen);
        self.caught = Self::clean_ids(&self.caught);
        // Caught implies seen
        for &id in &self.caught.clone() {
            if !self.seen.contains(&id) {
                self.seen.push(id);
            }
        }
        self.seen = Self::clean_ids(&self.seen);
    }

    fn clean_ids(ids: &[u16]) -> Vec<u16> {
        let mut out = Vec::new();
        for &id in ids {
            if Self::valid_id(id) && !out.contains(&id) {
                out.push(id);
            }
        }
        out.truncate(DEX_SIZE as usize);
        out
    }

    pub fn seen_count(&self) -> usize {
        self.seen
            .iter()
            .filter(|&&id| Self::valid_id(id))
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    pub fn caught_count(&self) -> usize {
        self.caught
            .iter()
            .filter(|&&id| Self::valid_id(id))
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    pub fn has_seen(&self, id: u16) -> bool {
        Self::valid_id(id) && self.seen.contains(&id)
    }

    pub fn has_caught(&self, id: u16) -> bool {
        Self::valid_id(id) && self.caught.contains(&id)
    }
}

/// Minimal save — only what the simplified game needs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveGame {
    pub version: u32,
    pub player_name: String,
    pub money: u32,
    pub party: Vec<PokemonInstance>,
    pub inventory: Vec<InventoryItem>,
    pub player_tx: i32,
    pub player_ty: i32,
    pub dex: DexProgress,
    pub starter_chosen: bool,
    pub battles_won: u32,
    pub pokemon_caught: u32,
}

impl Default for SaveGame {
    fn default() -> Self {
        Self {
            version: 3,
            player_name: "Trainer".into(),
            money: 300,
            party: vec![],
            inventory: vec![
                InventoryItem {
                    kind: ItemKind::PokeBall,
                    count: 8,
                },
                InventoryItem {
                    kind: ItemKind::Potion,
                    count: 5,
                },
            ],
            player_tx: 8,
            player_ty: 10,
            dex: DexProgress::default(),
            starter_chosen: false,
            battles_won: 0,
            pokemon_caught: 0,
        }
    }
}

impl SaveGame {
    pub fn item_count(&self, kind: ItemKind) -> u32 {
        self.inventory
            .iter()
            .find(|i| i.kind == kind)
            .map(|i| i.count)
            .unwrap_or(0)
    }

    pub fn add_item(&mut self, kind: ItemKind, n: u32) {
        if let Some(it) = self.inventory.iter_mut().find(|i| i.kind == kind) {
            it.count = it.count.saturating_add(n);
        } else {
            self.inventory.push(InventoryItem { kind, count: n });
        }
    }

    pub fn consume_item(&mut self, kind: ItemKind) -> bool {
        if let Some(it) = self.inventory.iter_mut().find(|i| i.kind == kind) {
            if it.count > 0 {
                it.count -= 1;
                return true;
            }
        }
        false
    }

    pub fn heal_party_full(&mut self) {
        for p in &mut self.party {
            p.heal_full();
        }
    }

    pub fn any_conscious(&self) -> bool {
        self.party.iter().any(|p| !p.is_fainted())
    }

    pub fn first_conscious_index(&self) -> Option<usize> {
        self.party.iter().position(|p| !p.is_fainted())
    }

    /// Species id for overworld follower (matches battle lead: first conscious).
    pub fn follower_species_id(&self) -> Option<u16> {
        self.first_conscious_index()
            .and_then(|i| self.party.get(i))
            .map(|p| p.species_id)
    }

    /// Clamp party, dex, money, coords; drop invalid species; fix starter flags.
    pub fn normalize(&mut self, map: &[Vec<Tile>]) {
        self.version = 3;
        self.player_name = clamp_chars(&self.player_name, 24);
        if self.player_name.is_empty() {
            self.player_name = "Trainer".into();
        }
        self.money = self.money.min(999_999);
        self.battles_won = self.battles_won.min(999_999);
        self.pokemon_caught = self.pokemon_caught.min(999_999);

        // Party: valid species only, max 6
        self.party.retain(|p| species_by_id(p.species_id).is_some());
        if self.party.len() > 6 {
            self.party.drain(6..);
        }
        for p in &mut self.party {
            if p.level == 0 {
                p.level = 1;
            }
            p.level = p.level.min(100);
            p.current_hp = p.current_hp.clamp(0, p.max_hp.max(1));
            p.nickname = clamp_chars(&p.nickname, 16);
        }

        self.dex.normalize();
        // Align catch counter with actual caught entries (floor, don't lower player progress below dex)
        let dex_caught = self.dex.caught_count() as u32;
        if self.pokemon_caught < dex_caught {
            self.pokemon_caught = dex_caught;
        }

        // Inventory: drop nonsense counts
        self.inventory.retain(|i| i.count > 0);
        for it in &mut self.inventory {
            it.count = it.count.min(999);
        }

        let (tx, ty) = normalize_spawn(self.player_tx, self.player_ty, map);
        self.player_tx = tx.clamp(0, MAP_W as i32 - 1);
        self.player_ty = ty.clamp(0, MAP_H as i32 - 1);

        if self.party.is_empty() {
            self.starter_chosen = false;
        } else {
            self.starter_chosen = true;
        }
    }
}

pub fn default_save_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pokemon-text-game")
        .join("save_v3.json")
}

pub fn read_save(path: &Path) -> Result<SaveGame, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > MAX_SAVE_BYTES {
        return Err("Save file too large".into());
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if data.len() as u64 > MAX_SAVE_BYTES {
        return Err("Save file too large".into());
    }
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

/// Load + normalize against map (call this from game after map is built).
pub fn read_save_normalized(path: &Path, map: &[Vec<Tile>]) -> Result<SaveGame, String> {
    let mut s = read_save(path)?;
    s.normalize(map);
    Ok(s)
}

pub fn write_save(path: &Path, save: &SaveGame) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| "Could not create save folder".to_string())?;
    }
    let data =
        serde_json::to_string_pretty(save).map_err(|_| "Could not encode save".to_string())?;
    fs::write(path, data).map_err(|_| "Could not write save file".to_string())
}

/// Capture probability roll (simplified Gen-style).
pub fn roll_capture(max_hp: i64, current_hp: i64, capture_rate: u8, ball_mod: f64) -> bool {
    if max_hp <= 0 || ball_mod <= 0.0 {
        return false;
    }
    let hp_factor = ((3 * max_hp - 2 * current_hp.max(1)) as f64 / (3 * max_hp) as f64).max(0.0);
    let a = (capture_rate as f64) * ball_mod * hp_factor;
    let chance = (a / 255.0).clamp(0.0, 1.0);
    rand::random::<f64>() < chance.max(0.08)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::build_map;

    #[test]
    fn item_add_consume() {
        let mut s = SaveGame::default();
        assert_eq!(s.item_count(ItemKind::PokeBall), 8);
        assert!(s.consume_item(ItemKind::PokeBall));
        assert_eq!(s.item_count(ItemKind::PokeBall), 7);
        s.add_item(ItemKind::PokeBall, 2);
        assert_eq!(s.item_count(ItemKind::PokeBall), 9);
    }

    #[test]
    fn dex_mark_and_normalize() {
        let mut d = DexProgress::default();
        d.mark_seen(25);
        d.mark_seen(25);
        d.mark_caught(25);
        d.mark_seen(0);
        d.mark_seen(999);
        d.seen.push(25);
        d.normalize();
        assert_eq!(d.seen_count(), 1);
        assert_eq!(d.caught_count(), 1);
        assert!(d.has_caught(25));
        assert!(!d.has_seen(0));
    }

    #[test]
    fn normalize_party_and_coords() {
        let m = build_map();
        let mut s = SaveGame::default();
        s.player_tx = -10;
        s.player_ty = 500;
        s.party
            .push(PokemonInstance::from_species_id(1, 5).unwrap());
        s.party
            .push(PokemonInstance::from_species_id(4, 5).unwrap());
        // Extra junk slots
        for _ in 0..10 {
            if let Some(p) = PokemonInstance::from_species_id(7, 5) {
                s.party.push(p);
            }
        }
        s.normalize(&m);
        assert!(s.party.len() <= 6);
        assert!(s.player_tx >= 0 && s.player_ty >= 0);
        assert!(s.starter_chosen);
    }

    #[test]
    fn full_party_catch_still_marks_dex() {
        // Documents intended behavior: counters increment even if party can't hold the mon.
        let mut s = SaveGame::default();
        for id in [1u16, 4, 7, 25, 10, 16] {
            s.party
                .push(PokemonInstance::from_species_id(id, 5).unwrap());
        }
        assert_eq!(s.party.len(), 6);
        s.dex.mark_caught(52);
        s.pokemon_caught += 1;
        // Mon not added — caller responsibility
        assert_eq!(s.party.len(), 6);
        assert!(s.dex.has_caught(52));
    }

    #[test]
    fn clamp_chars_utf8_safe() {
        // Multi-byte Japanese: byte truncate(24) would panic mid-codepoint.
        let long = "ポケモン冒険家トレーナー名前テスト用";
        let clamped = clamp_chars(long, 8);
        assert_eq!(clamped.chars().count(), 8);
        assert!(!clamped.is_empty());
    }

    #[test]
    fn normalize_multibyte_names_no_panic() {
        let m = build_map();
        let mut s = SaveGame::default();
        s.player_name = "トレーナー名超長い名前ですよ本当に".into();
        let mut p = PokemonInstance::from_species_id(25, 5).unwrap();
        p.nickname = "ピカチュウ大好きニックネーム".into();
        s.party.push(p);
        s.normalize(&m); // must not panic
        assert!(s.player_name.chars().count() <= 24);
        assert!(s.party[0].nickname.chars().count() <= 16);
    }

    #[test]
    fn read_save_rejects_oversize() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("huge.json");
        let mut f = std::fs::File::create(&path).unwrap();
        let chunk = vec![b'x'; 8192];
        let mut written = 0u64;
        while written <= MAX_SAVE_BYTES {
            f.write_all(&chunk).unwrap();
            written += chunk.len() as u64;
        }
        drop(f);
        let err = read_save(&path).unwrap_err();
        assert!(err.contains("large") || err.contains("Large"));
    }
}
