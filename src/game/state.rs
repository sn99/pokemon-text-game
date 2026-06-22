//! Main game state machine.

use macroquad::prelude::*;

use crate::battle::engine::execute_move;
use crate::pokemon::species::species_by_id;
use crate::pokemon::stats::{xp_for_level, xp_gain_from_defeat};
use crate::pokemon::PokemonInstance;
use crate::save::{
    default_save_path, read_save_normalized, roll_capture, write_save, ItemKind, SaveGame, DEX_SIZE,
};
use crate::world::{
    area_name_for_tile, build_map, interact_prop_at, is_npc_at, starter_species,
    wild_pool_for_tile, Tile, WorldProp, MAP_H, MAP_W, TILE_PX,
};

use super::assets::SpriteCache;
use super::battle_ui::{
    draw_bag_menu, draw_battle_backdrop, draw_battle_log, draw_battle_menu, draw_float_texts,
    draw_overlay_banner, draw_pokemon_sprite, draw_switch_menu, FloatText,
};
use super::overworld::{
    draw_dialogue_box, draw_follower, draw_hud_bar, draw_map, draw_player, draw_toast,
    draw_world_props,
};
use super::theme::{
    draw_gradient_bg, draw_modal_dim, draw_panel, draw_select_row, draw_status_box,
    draw_title_text, draw_vignette, C_ACCENT, C_TEXT, C_TEXT_DIM,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    Starter,
    Overworld,
    Battle,
    Party,
    Pause,
    Shop,
    Dialogue,
    Pokedex,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BattlePhase {
    PlayerPick,
    PlayerMoves,
    BagMenu,
    SwitchMenu,
    FoeTurn,
    Won,
    Lost,
    Caught,
    Fled,
}

struct BattleRuntime {
    player_idx: usize,
    player: PokemonInstance,
    foe: PokemonInstance,
    is_wild: bool,
    log: Vec<String>,
    phase: BattlePhase,
    menu_idx: usize,
    move_idx: usize,
    bag_idx: usize,
    switch_idx: usize,
    wait_timer: f32,
    shake_foe: f32,
    shake_player: f32,
    /// Intro slide / flash timer before player can act.
    intro_t: f32,
    floats: Vec<FloatText>,
    hit_flash_foe: f32,
    hit_flash_player: f32,
    /// Smoothed HP shown in status bars (lerps toward real HP).
    disp_hp_foe: f32,
    disp_hp_player: f32,
}

/// Brief visual fleck when walking through tall grass.
struct GrassFleck {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

pub struct Game {
    sprites: SpriteCache,
    save: SaveGame,
    map: Vec<Vec<Tile>>,
    screen: Screen,
    /// Rendered pixel position (lerped toward target).
    px: f32,
    py: f32,
    /// Grid-aligned walk target in pixels.
    target_px: f32,
    target_py: f32,
    facing: u8, // 0 down 1 left 2 right 3 up
    walk_cd: f32,
    walk_phase: f32,
    moving: bool,
    title_idx: usize,
    starter_idx: usize,
    party_idx: usize,
    pause_idx: usize,
    shop_idx: usize,
    toast: String,
    toast_timer: f32,
    battle: Option<BattleRuntime>,
    steps_since_enc: u32,
    /// Screen flash when entering battle (0 = off).
    battle_flash: f32,
    /// Auto-save cooldown so we don't spam writes.
    autosave_cd: f32,
    /// Smoothed camera position.
    cam_x: f32,
    cam_y: f32,
    cam_ready: bool,
    /// Buffered walk direction while mid-slide (dx, dy).
    pending_step: Option<(i32, i32)>,
    /// Particles when stepping in tall grass.
    grass_flecks: Vec<GrassFleck>,
    /// Active dialogue (sign / NPC).
    dialogue_prop: Option<&'static WorldProp>,
    dialogue_line: usize,
    /// Pokédex scroll index (species id 1..=151, 0-based row offset).
    dex_scroll: usize,
    dex_sel: usize,
    /// When true, Esc from Party returns to Pause instead of Overworld.
    party_from_pause: bool,
}

impl Game {
    pub async fn new() -> Self {
        let sprites = SpriteCache::load_with_progress().await;
        let map = build_map();
        let path = default_save_path();
        let mut save = read_save_normalized(&path, &map).unwrap_or_default();
        save.normalize(&map);
        let (px, py) = (
            save.player_tx as f32 * TILE_PX,
            save.player_ty as f32 * TILE_PX,
        );
        let screen = if save.starter_chosen && !save.party.is_empty() {
            Screen::Overworld
        } else {
            Screen::Title
        };
        Self {
            sprites,
            save,
            map,
            screen,
            px,
            py,
            target_px: px,
            target_py: py,
            facing: 0,
            walk_cd: 0.0,
            walk_phase: 0.0,
            moving: false,
            title_idx: 0,
            starter_idx: 0,
            party_idx: 0,
            pause_idx: 0,
            shop_idx: 0,
            toast: String::new(),
            toast_timer: 0.0,
            battle: None,
            steps_since_enc: 0,
            battle_flash: 0.0,
            autosave_cd: 0.0,
            cam_x: 0.0,
            cam_y: 0.0,
            cam_ready: false,
            pending_step: None,
            grass_flecks: Vec::new(),
            dialogue_prop: None,
            dialogue_line: 0,
            dex_scroll: 0,
            dex_sel: 0,
            party_from_pause: false,
        }
    }

    fn spawn_grass_flecks(&mut self) {
        let (tx, ty) = self.grid_pos();
        let base_x = tx as f32 * TILE_PX + TILE_PX * 0.5;
        let base_y = ty as f32 * TILE_PX + TILE_PX * 0.65;
        for i in 0..5 {
            let ang = i as f32 * 1.1 + ::rand::random::<f32>() * 0.6;
            self.grass_flecks.push(GrassFleck {
                x: base_x + (::rand::random::<f32>() - 0.5) * 10.0,
                y: base_y,
                vx: ang.cos() * 28.0,
                vy: -35.0 - ::rand::random::<f32>() * 25.0,
                life: 0.45 + ::rand::random::<f32>() * 0.2,
            });
        }
        if self.grass_flecks.len() > 40 {
            let n = self.grass_flecks.len() - 40;
            self.grass_flecks.drain(0..n);
        }
    }

    fn tick_grass_flecks(&mut self, dt: f32) {
        for f in &mut self.grass_flecks {
            f.life -= dt;
            f.x += f.vx * dt;
            f.y += f.vy * dt;
            f.vy += 90.0 * dt;
        }
        self.grass_flecks.retain(|f| f.life > 0.0);
    }

    fn apply_loaded_save(&mut self, mut save: SaveGame) {
        save.normalize(&self.map);
        self.save = save;
        self.px = self.save.player_tx as f32 * TILE_PX;
        self.py = self.save.player_ty as f32 * TILE_PX;
        self.target_px = self.px;
        self.target_py = self.py;
        self.moving = false;
        self.pending_step = None;
        self.walk_cd = 0.0;
        self.walk_phase = 0.0;
        self.cam_ready = false;
        self.cam_x = 0.0;
        self.cam_y = 0.0;
        self.battle = None;
        self.battle_flash = 0.0;
        self.grass_flecks.clear();
        self.dialogue_prop = None;
        self.dialogue_line = 0;
        self.party_from_pause = false;
        self.toast.clear();
        self.toast_timer = 0.0;
    }

    fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast = msg.into();
        self.toast_timer = 2.5;
    }

    fn try_save(&mut self) {
        self.save.player_tx = (self.target_px / TILE_PX).round() as i32;
        self.save.player_ty = (self.target_py / TILE_PX).round() as i32;
        self.save.normalize(&self.map);
        let path = default_save_path();
        match write_save(&path, &self.save) {
            Ok(()) => self.show_toast("Game saved!"),
            Err(_) => self.show_toast("Save failed — check disk permissions."),
        }
    }

    fn quiet_autosave(&mut self) {
        if self.autosave_cd > 0.0 {
            return;
        }
        self.save.player_tx = (self.target_px / TILE_PX).round() as i32;
        self.save.player_ty = (self.target_py / TILE_PX).round() as i32;
        self.save.normalize(&self.map);
        let path = default_save_path();
        let _ = write_save(&path, &self.save);
        self.autosave_cd = 45.0; // at most once per 45s of play
    }

    fn tile_at(&self, tx: i32, ty: i32) -> Option<Tile> {
        if tx < 0 || ty < 0 || tx >= MAP_W as i32 || ty >= MAP_H as i32 {
            return None;
        }
        Some(self.map[ty as usize][tx as usize])
    }

    fn grid_pos(&self) -> (i32, i32) {
        (
            (self.target_px / TILE_PX).round() as i32,
            (self.target_py / TILE_PX).round() as i32,
        )
    }

    fn try_move(&mut self, dx: i32, dy: i32) {
        // Buffer next step while sliding so hold-to-walk feels responsive
        if self.moving || self.walk_cd > 0.0 {
            self.pending_step = Some((dx, dy));
            // Still update facing immediately for feedback
            if dx < 0 {
                self.facing = 1;
            } else if dx > 0 {
                self.facing = 2;
            } else if dy < 0 {
                self.facing = 3;
            } else if dy > 0 {
                self.facing = 0;
            }
            return;
        }
        if dx < 0 {
            self.facing = 1;
        } else if dx > 0 {
            self.facing = 2;
        } else if dy < 0 {
            self.facing = 3;
        } else if dy > 0 {
            self.facing = 0;
        }

        let (cur_tx, cur_ty) = self.grid_pos();
        let ntx = cur_tx + dx;
        let nty = cur_ty + dy;
        let Some(tile) = self.tile_at(ntx, nty) else {
            return;
        };
        if !tile.walkable() || is_npc_at(ntx, nty) {
            return;
        }

        self.target_px = ntx as f32 * TILE_PX;
        self.target_py = nty as f32 * TILE_PX;
        self.moving = true;
        self.pending_step = None;
        self.walk_cd = 0.02;
        self.walk_phase += 1.0;
        self.steps_since_enc += 1;

        // Stepping onto center door/floor = auto heal prompt
        if tile.is_center_area() || tile == Tile::Door {
            // subtle: only full heal if HP missing or on door
            let needs = self
                .save
                .party
                .iter()
                .any(|p| p.current_hp < p.max_hp || p.is_fainted());
            if needs && tile == Tile::Door {
                self.save.heal_party_full();
                self.show_toast("Welcome! Your Pokémon were healed.");
                self.quiet_autosave();
            }
        }

        if tile.is_tall_grass() {
            self.spawn_grass_flecks();
            if !self.save.any_conscious() {
                if self.steps_since_enc % 8 == 1 {
                    self.show_toast("All fainted! Heal at the Center (red building).");
                }
            } else if self.steps_since_enc > 2 && (::rand::random::<u8>() % 100) < 16 {
                // ~16% encounter after a few steps
                self.steps_since_enc = 0;
                self.start_wild_battle(ntx, nty);
            }
        }

        // Auto-open mart when stepping on mart door
        if tile.is_mart_area() {
            self.shop_idx = 0;
            self.screen = Screen::Shop;
        }
    }

    fn tick_movement(&mut self, dt: f32) {
        if !self.moving {
            // Snap cleanly when idle
            self.px = self.target_px;
            self.py = self.target_py;
            return;
        }
        let speed = TILE_PX * 9.0; // tiles per second-ish
        let dx = self.target_px - self.px;
        let dy = self.target_py - self.py;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 {
            self.px = self.target_px;
            self.py = self.target_py;
            self.moving = false;
            // Consume buffered step on arrival
            if let Some((dx, dy)) = self.pending_step.take() {
                self.try_move(dx, dy);
            }
            return;
        }
        let step = speed * dt;
        let t = (step / dist).min(1.0);
        self.px += dx * t;
        self.py += dy * t;
        self.walk_phase += dt * 14.0;
    }

    fn poke_speed(inst: &PokemonInstance) -> i64 {
        species_by_id(inst.species_id)
            .map(|s| s.base_stats.speed_at_level(inst.level))
            .unwrap_or(50)
            + (inst.iv_speed / 4) as i64
    }

    fn party_lead_level(&self) -> u8 {
        self.save
            .first_conscious_index()
            .and_then(|i| self.save.party.get(i))
            .map(|p| p.level)
            .unwrap_or(5)
    }

    fn xp_progress(inst: &PokemonInstance) -> f32 {
        if inst.level >= 100 {
            return 1.0;
        }
        let cur_floor = xp_for_level(inst.level);
        let next = xp_for_level(inst.level + 1);
        if next <= cur_floor {
            return 1.0;
        }
        let into = inst.experience.saturating_sub(cur_floor) as f32;
        let span = (next - cur_floor) as f32;
        (into / span).clamp(0.0, 1.0)
    }

    fn start_wild_battle(&mut self, tx: i32, ty: i32) {
        if self.save.party.is_empty() || !self.save.any_conscious() {
            self.show_toast("No battle-ready Pokémon! Heal at the Center.");
            return;
        }
        let (lo, hi, pool) = wild_pool_for_tile(tx, ty);
        // Only species that exist in the DB (avoid silent encounter failures)
        let valid: Vec<u16> = pool
            .iter()
            .copied()
            .filter(|&id| species_by_id(id).is_some())
            .collect();
        let sid = if valid.is_empty() {
            // Last resort: Rattata / Pidgey / Caterpie fallbacks
            let fallbacks = [19u16, 16, 10, 1];
            let Some(&fb) = fallbacks.iter().find(|&&id| species_by_id(id).is_some()) else {
                self.show_toast("No wild data loaded.");
                return;
            };
            fb
        } else {
            valid[(::rand::random::<usize>()) % valid.len()]
        };
        // Scale wild levels toward party lead so late game isn't trivial/impossible
        let lead = self.party_lead_level();
        let adj_lo = lo.max(lead.saturating_sub(3)).min(98);
        let adj_hi = hi.max(lead.saturating_sub(1)).min(100).max(adj_lo);
        let span = (adj_hi - adj_lo + 1) as u8;
        let lvl = adj_lo + (::rand::random::<u8>() % span.max(1));
        let Some(foe) = PokemonInstance::from_species_id(sid, lvl) else {
            self.show_toast("Encounter failed — missing species data.");
            return;
        };
        let Some(pidx) = self.save.first_conscious_index() else {
            return;
        };
        let Some(player) = self.save.party.get(pidx).cloned() else {
            return;
        };
        self.save.dex.mark_seen(sid);

        let name = foe.display_name().to_string();
        self.battle_flash = 0.55;
        // Keep walking anim from carrying weird phase into battle return
        self.moving = false;
        self.px = self.target_px;
        self.py = self.target_py;
        let foe_hp = foe.current_hp as f32;
        let pl_hp = player.current_hp as f32;
        self.battle = Some(BattleRuntime {
            player_idx: pidx,
            player,
            foe,
            is_wild: true,
            log: vec![format!("A wild {} appeared!", name)],
            phase: BattlePhase::PlayerPick,
            menu_idx: 0,
            move_idx: 0,
            bag_idx: 0,
            switch_idx: 0,
            wait_timer: 0.0,
            shake_foe: 0.0,
            shake_player: 0.0,
            intro_t: 0.85,
            floats: Vec::new(),
            hit_flash_foe: 0.0,
            hit_flash_player: 0.0,
            disp_hp_foe: foe_hp,
            disp_hp_player: pl_hp,
        });
        self.screen = Screen::Battle;
        let _ = (tx, ty);
    }

    fn sync_player_from_save(&mut self) {
        if let Some(b) = &mut self.battle {
            if b.player_idx < self.save.party.len() {
                self.save.party[b.player_idx] = b.player.clone();
            }
        }
    }

    fn apply_xp_to_player(&mut self, foe: &PokemonInstance) {
        let Some(b) = &self.battle else { return };
        let pidx = b.player_idx;
        let base_yield = species_by_id(foe.species_id)
            .map(|s| s.base_experience.max(1))
            .unwrap_or(64);
        let gain = xp_gain_from_defeat(foe.level, base_yield);
        if pidx >= self.save.party.len() {
            return;
        }
        let old_level = self.save.party[pidx].level;
        let msgs = {
            let p = &mut self.save.party[pidx];
            p.gain_xp(gain)
        };
        if let Some(b) = &mut self.battle {
            for m in msgs {
                b.log.push(m);
            }
            b.player = self.save.party[pidx].clone();
        }
        if self.save.party[pidx].level > old_level {
            let evo_msgs = self.save.party[pidx].try_evolve();
            if let Some(b) = &mut self.battle {
                for m in evo_msgs {
                    b.log.push(m);
                }
                if pidx < self.save.party.len() {
                    b.player = self.save.party[pidx].clone();
                }
            }
        }
    }

    fn end_battle_to_overworld(&mut self) {
        self.sync_player_from_save();
        self.battle = None;
        self.screen = Screen::Overworld;
        if !self.save.any_conscious() {
            // Rescue at center
            self.save.heal_party_full();
            self.px = 7.0 * TILE_PX;
            self.py = 19.0 * TILE_PX;
            self.target_px = self.px;
            self.target_py = self.py;
            self.show_toast("You blacked out... Healed at the Center.");
            self.quiet_autosave();
        }
    }

    fn apply_player_move_result(&mut self, move_index: usize) -> bool {
        let Some(b) = &mut self.battle else {
            return false;
        };
        let result = execute_move(&mut b.player, &mut b.foe, move_index);
        let pname = b.player.display_name().to_string();
        let mname = if move_index < b.player.moves.len() {
            b.player.moves[move_index].data.name.clone()
        } else {
            "?".into()
        };

        let w = screen_width();
        let h = screen_height();
        if result.missed {
            b.log
                .push(format!("{} used {}... but it missed!", pname, mname));
        } else {
            let mut line = format!("{} used {}! (-{})", pname, mname, result.damage_dealt);
            if result.critical {
                line.push_str(" Critical hit!");
            }
            if !result.effectiveness_text.is_empty()
                && result.effectiveness != 1.0
                && result.effectiveness > 0.0
            {
                line.push(' ');
                line.push_str(&result.effectiveness_text);
            }
            if result.effectiveness == 0.0 {
                line = format!("{} used {}! It had no effect...", pname, mname);
            }
            b.log.push(line);
            b.shake_foe = 10.0;
            if result.damage_dealt > 0 {
                b.hit_flash_foe = 0.22;
                b.floats
                    .push(FloatText::damage(w * 0.62, h * 0.22, result.damage_dealt));
            }
        }

        if b.foe.is_fainted() {
            b.log.push(format!("{} fainted!", b.foe.display_name()));
            b.phase = BattlePhase::Won;
            b.wait_timer = 1.2;
            return true; // battle over
        }
        false
    }

    fn player_attack(&mut self, move_index: usize) {
        let (p_spd, f_spd) = {
            let Some(b) = &self.battle else { return };
            (Self::poke_speed(&b.player), Self::poke_speed(&b.foe))
        };

        // Faster pokemon goes first (player wins ties — standard favor)
        let player_first = p_spd >= f_spd;

        if player_first {
            if self.apply_player_move_result(move_index) {
                return;
            }
            if let Some(b) = &mut self.battle {
                b.phase = BattlePhase::FoeTurn;
                b.wait_timer = 0.48;
            }
        } else {
            // Foe strikes first this turn, then player resolves move
            if let Some(b) = &mut self.battle {
                b.log.push(format!("{} is faster!", b.foe.display_name()));
            }
            self.foe_attack();
            // If battle ended or player fainted handled in foe_attack; if still in player pick, do our move
            let still_fighting = self
                .battle
                .as_ref()
                .map(|b| {
                    matches!(b.phase, BattlePhase::PlayerPick | BattlePhase::FoeTurn)
                        && !b.player.is_fainted()
                        && !b.foe.is_fainted()
                })
                .unwrap_or(false);
            if still_fighting {
                if self.apply_player_move_result(move_index) {
                    return;
                }
                if let Some(b) = &mut self.battle {
                    b.phase = BattlePhase::PlayerPick;
                }
            }
        }
    }

    fn foe_attack(&mut self) {
        let Some(b) = &mut self.battle else { return };
        let usable: Vec<usize> = b
            .foe
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| m.can_use())
            .map(|(i, _)| i)
            .collect();
        if usable.is_empty() {
            b.log.push(format!("{} struggles...", b.foe.display_name()));
            b.phase = BattlePhase::PlayerPick;
            return;
        }
        let mi = usable[(::rand::random::<usize>()) % usable.len()];
        let result = execute_move(&mut b.foe, &mut b.player, mi);
        let fname = b.foe.display_name().to_string();
        let mname = b.foe.moves[mi].data.name.clone();

        let w = screen_width();
        let h = screen_height();
        if result.missed {
            b.log
                .push(format!("{} used {}... but it missed!", fname, mname));
        } else {
            let mut line = format!("{} used {}! (-{})", fname, mname, result.damage_dealt);
            if result.critical {
                line.push_str(" Critical!");
            }
            if result.effectiveness == 0.0 {
                line = format!("{} used {}! No effect...", fname, mname);
            }
            b.log.push(line);
            b.shake_player = 8.0;
            if result.damage_dealt > 0 {
                b.hit_flash_player = 0.22;
                b.floats
                    .push(FloatText::damage(w * 0.22, h * 0.48, result.damage_dealt));
            }
        }

        if b.player.is_fainted() {
            b.log.push(format!("{} fainted!", b.player.display_name()));
            // try switch
            if let Some(idx) = self.save.party.iter().position(|p| !p.is_fainted()) {
                if idx != b.player_idx {
                    // save fainted mon first
                    if b.player_idx < self.save.party.len() {
                        self.save.party[b.player_idx] = b.player.clone();
                    }
                    b.player_idx = idx;
                    b.player = self.save.party[idx].clone();
                    b.log.push(format!("Go! {}!", b.player.display_name()));
                    b.phase = BattlePhase::PlayerPick;
                    return;
                }
            }
            // all fainted
            if b.player_idx < self.save.party.len() {
                self.save.party[b.player_idx] = b.player.clone();
            }
            b.phase = BattlePhase::Lost;
            b.wait_timer = 1.5;
            return;
        }

        b.phase = BattlePhase::PlayerPick;
    }

    fn estimate_catch_pct(foe: &PokemonInstance) -> u32 {
        let rate = species_by_id(foe.species_id)
            .map(|s| s.capture_rate)
            .unwrap_or(45) as f64;
        let max_hp = foe.max_hp.max(1) as f64;
        let cur = foe.current_hp.max(1) as f64;
        let hp_factor = ((3.0 * max_hp - 2.0 * cur) / (3.0 * max_hp)).max(0.0);
        let a = rate * 1.0 * hp_factor;
        let chance = (a / 255.0).clamp(0.0, 1.0).max(0.08);
        (chance * 100.0).round() as u32
    }

    fn try_catch(&mut self) {
        let balls = self.save.item_count(ItemKind::PokeBall);
        if balls == 0 {
            if let Some(b) = &mut self.battle {
                b.log.push("No Poke Balls left!".into());
                b.phase = BattlePhase::PlayerPick;
            }
            return;
        }
        let Some(b) = &mut self.battle else { return };
        if !b.is_wild {
            b.log.push("Can't catch trainer Pokemon!".into());
            b.phase = BattlePhase::PlayerPick;
            return;
        }
        let est = Self::estimate_catch_pct(&b.foe);
        self.save.consume_item(ItemKind::PokeBall);
        let rate = species_by_id(b.foe.species_id)
            .map(|s| s.capture_rate)
            .unwrap_or(45);
        let ok = roll_capture(b.foe.max_hp, b.foe.current_hp, rate, 1.0);
        if ok {
            let caught = b.foe.clone();
            let name = caught.display_name().to_string();
            let sid = caught.species_id;
            b.log
                .push(format!("Gotcha! {} was caught! (~{}% odds)", name, est));
            self.save.dex.mark_caught(sid);
            self.save.pokemon_caught += 1;
            self.save.money += 15; // small capture bonus
            if self.save.party.len() < 6 {
                self.save.party.push(caught);
            } else {
                b.log
                    .push("Party full — couldn't keep it (release for now).".into());
            }
            b.phase = BattlePhase::Caught;
            b.wait_timer = 1.4;
        } else {
            b.log.push(format!(
                "Oh no! It broke free! (was ~{}% — try lowering HP)",
                est
            ));
            b.phase = BattlePhase::FoeTurn;
            b.wait_timer = 0.45;
        }
    }

    fn try_potion_in_battle(&mut self) {
        if self.save.item_count(ItemKind::Potion) == 0 {
            if let Some(b) = &mut self.battle {
                b.log.push("No Potions left!".into());
                b.phase = BattlePhase::PlayerPick;
            }
            return;
        }
        let Some(b) = &mut self.battle else { return };
        if b.player.current_hp >= b.player.max_hp {
            b.log.push("HP is already full!".into());
            b.phase = BattlePhase::PlayerPick;
            return;
        }
        self.save.consume_item(ItemKind::Potion);
        let heal = ItemKind::Potion.heal_amount();
        let before = b.player.current_hp;
        b.player.current_hp = (b.player.current_hp + heal).min(b.player.max_hp);
        let actual = b.player.current_hp - before;
        b.log.push(format!("Used Potion! Restored {} HP.", actual));
        let w = screen_width();
        let h = screen_height();
        b.floats.push(FloatText::heal(w * 0.22, h * 0.48, actual));
        b.phase = BattlePhase::FoeTurn;
        b.wait_timer = 0.45;
    }

    fn try_switch_in_battle(&mut self, party_i: usize) {
        let Some(b) = &mut self.battle else { return };
        if party_i >= self.save.party.len() || party_i == b.player_idx {
            b.log.push("Can't switch to that!".into());
            b.phase = BattlePhase::PlayerPick;
            return;
        }
        if self.save.party[party_i].is_fainted() {
            b.log.push("That Pokémon has fainted!".into());
            b.phase = BattlePhase::PlayerPick;
            return;
        }
        // Save current fighter
        if b.player_idx < self.save.party.len() {
            self.save.party[b.player_idx] = b.player.clone();
        }
        b.player_idx = party_i;
        b.player = self.save.party[party_i].clone();
        let name = b.player.display_name().to_string();
        b.log.push(format!("Go! {}!", name));
        b.phase = BattlePhase::FoeTurn;
        b.wait_timer = 0.45;
    }

    fn open_dialogue(&mut self, prop: &'static WorldProp) {
        self.dialogue_prop = Some(prop);
        self.dialogue_line = 0;
        self.screen = Screen::Dialogue;
    }

    fn try_interact(&mut self) {
        // Center/Mart service first so E near door heals/shops (HUD: Talk/Heal/Shop).
        if self.try_service_interact() {
            return;
        }
        let (tx, ty) = self.grid_pos();
        if let Some(prop) = interact_prop_at(tx, ty, self.facing) {
            self.open_dialogue(prop);
            return;
        }
        self.show_toast("E/H: face signs/NPCs · stand on Center door/floor · Mart door");
    }

    /// Heal at Center or open Mart when on/near service tiles (or facing door).
    /// Checked **before** prop dialogue so E at the Center always heals.
    fn try_service_interact(&mut self) -> bool {
        let (tx, ty) = self.grid_pos();
        let (fx, fy) = match self.facing {
            1 => (-1, 0),
            2 => (1, 0),
            3 => (0, -1),
            _ => (0, 1),
        };
        let tile_here = self.tile_at(tx, ty);
        let tile_front = self.tile_at(tx + fx, ty + fy);

        let mart_service = tile_here.map(|t| t.is_mart_area()).unwrap_or(false)
            || tile_front
                .map(|t| t.is_mart_area() || t == Tile::Mart)
                .unwrap_or(false)
            || (-1i32..=1).any(|dy| {
                (-1i32..=1).any(|dx| {
                    self.tile_at(tx + dx, ty + dy)
                        .map(|t| t.is_mart_area())
                        .unwrap_or(false)
                })
            });
        if mart_service {
            self.shop_idx = 0;
            self.screen = Screen::Shop;
            return true;
        }

        let center_service = tile_here.map(|t| t.is_center_area()).unwrap_or(false)
            || tile_front
                .map(|t| t.is_center_area() || t == Tile::Door)
                .unwrap_or(false)
            || (tx >= 5 && tx <= 9 && ty >= 18 && ty <= 21);
        if center_service {
            self.save.heal_party_full();
            if self.save.item_count(ItemKind::PokeBall) < 3 {
                self.save.add_item(ItemKind::PokeBall, 2);
            }
            if self.save.item_count(ItemKind::Potion) < 2 {
                self.save.add_item(ItemKind::Potion, 1);
            }
            self.show_toast("Your Pokémon were fully healed!");
            self.quiet_autosave();
            return true;
        }
        false
    }

    fn try_buy(&mut self, which: usize) {
        // 0 = ball $100, 1 = potion $80, 2 = 3x balls $270, 3 = leave
        match which {
            0 => {
                if self.save.money >= 100 {
                    self.save.money -= 100;
                    self.save.add_item(ItemKind::PokeBall, 1);
                    self.show_toast("Bought Poke Ball!");
                } else {
                    self.show_toast("Not enough money ($100).");
                }
            }
            1 => {
                if self.save.money >= 80 {
                    self.save.money -= 80;
                    self.save.add_item(ItemKind::Potion, 1);
                    self.show_toast("Bought Potion!");
                } else {
                    self.show_toast("Not enough money ($80).");
                }
            }
            2 => {
                if self.save.money >= 270 {
                    self.save.money -= 270;
                    self.save.add_item(ItemKind::PokeBall, 3);
                    self.show_toast("Bought 3 Poke Balls!");
                } else {
                    self.show_toast("Not enough money ($270).");
                }
            }
            _ => {
                self.screen = Screen::Overworld;
            }
        }
    }

    fn update_shop(&mut self) {
        self.toast_timer = (self.toast_timer - get_frame_time()).max(0.0);
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Overworld;
            return;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.shop_idx = self.shop_idx.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.shop_idx = (self.shop_idx + 1).min(3);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.try_buy(self.shop_idx);
        }
    }

    fn draw_shop(&self) {
        // Dim overworld beneath
        self.draw_overworld();
        let w = screen_width();
        let h = screen_height();
        draw_modal_dim(0.58);

        let mw = 420.0;
        let mh = 340.0;
        let mx = w / 2.0 - mw / 2.0;
        let my = h / 2.0 - mh / 2.0;
        draw_panel(mx, my, mw, mh, true);
        draw_rectangle(mx + 8.0, my + 8.0, mw - 16.0, 4.0, C_ACCENT);
        draw_title_text("Poké Mart", mx + 120.0, my + 44.0, 30.0);
        draw_text(
            &format!("Your money: ${}", self.save.money),
            mx + 24.0,
            my + 72.0,
            16.0,
            C_ACCENT,
        );
        draw_text(
            &format!(
                "Inventory: {} balls · {} potions",
                self.save.item_count(ItemKind::PokeBall),
                self.save.item_count(ItemKind::Potion)
            ),
            mx + 24.0,
            my + 92.0,
            14.0,
            C_TEXT_DIM,
        );

        let opts = [
            ("Poke Ball", "$100", "Catch wild Pokémon"),
            ("Potion", "$80", "Restore 40 HP"),
            ("Ball ×3", "$270", "Save $30 vs singles"),
            ("Leave", "—", "Back to overworld"),
        ];
        for (i, (name, price, sub)) in opts.iter().enumerate() {
            let y = my + 120.0 + i as f32 * 44.0;
            draw_select_row(mx + 14.0, y - 14.0, mw - 28.0, 42.0, i == self.shop_idx);
            draw_text(name, mx + 32.0, y + 4.0, 20.0, C_TEXT);
            draw_text(price, mx + mw - 96.0, y + 4.0, 18.0, C_ACCENT);
            draw_text(sub, mx + 32.0, y + 22.0, 13.0, C_TEXT_DIM);
        }
        draw_text(
            "↑↓ select · Enter buy · Esc leave",
            mx + 24.0,
            my + mh - 20.0,
            13.0,
            C_TEXT_DIM,
        );
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_title(&mut self) {
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.title_idx = self.title_idx.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.title_idx = (self.title_idx + 1).min(2);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match self.title_idx {
                0 => {
                    if self.save.starter_chosen && !self.save.party.is_empty() {
                        self.screen = Screen::Overworld;
                    } else {
                        self.screen = Screen::Starter;
                    }
                }
                1 => {
                    let path = default_save_path();
                    if let Ok(s) = read_save_normalized(&path, &self.map) {
                        self.apply_loaded_save(s);
                        if self.save.starter_chosen && !self.save.party.is_empty() {
                            self.screen = Screen::Overworld;
                            self.show_toast("Save loaded!");
                        } else {
                            self.screen = Screen::Starter;
                        }
                    } else {
                        self.show_toast("No save found — starting new game.");
                        self.screen = Screen::Starter;
                    }
                }
                _ => std::process::exit(0),
            }
        }
    }

    fn draw_title(&self) {
        let w = screen_width();
        let h = screen_height();
        let t = get_time() as f32;
        draw_gradient_bg(
            Color::new(0.07, 0.10, 0.22, 1.0),
            Color::new(0.12, 0.22, 0.38, 1.0),
            28,
        );
        // Floating accent orbs
        for i in 0..5 {
            let ox = (w * 0.15 * i as f32 + t * (8.0 + i as f32 * 2.0)) % (w + 40.0) - 20.0;
            let oy = 60.0 + (t * 0.7 + i as f32).sin() * 30.0 + i as f32 * 40.0;
            draw_circle(
                ox,
                oy,
                18.0 + i as f32 * 4.0,
                Color::new(0.3, 0.5, 0.9, 0.06),
            );
        }
        draw_vignette();

        let title_bob = (t * 1.4).sin() * 3.0;
        draw_title_text("POKÉMON 2D", w / 2.0 - 150.0, 82.0 + title_bob, 52.0);
        draw_text(
            "Walk · Talk · Battle · Catch · Grow   ·   v3.2",
            w / 2.0 - 155.0,
            118.0 + title_bob,
            18.0,
            C_TEXT_DIM,
        );

        // Sprite showcase on a platform panel
        draw_panel(40.0, 148.0, w - 80.0, 150.0, false);
        draw_rectangle(
            50.0,
            278.0,
            w - 100.0,
            12.0,
            Color::new(0.0, 0.0, 0.0, 0.25),
        );
        let ids = [25u16, 6, 9, 3, 150];
        for (i, id) in ids.iter().enumerate() {
            let x = 70.0 + i as f32 * ((w - 140.0) / 5.0);
            let bob = ((t * 2.0 + i as f32 * 0.9).sin()) * 4.0;
            draw_ellipse(
                x + 55.0,
                272.0,
                34.0,
                8.0,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.28),
            );
            draw_pokemon_sprite(&self.sprites, *id, x, 154.0 + bob, 110.0, false, 0.0);
        }

        let mw = 380.0;
        let mx = w / 2.0 - mw / 2.0;
        let my = 328.0;
        draw_panel(mx, my, mw, 178.0, true);
        draw_rectangle(mx + 10.0, my + 10.0, mw - 20.0, 3.0, C_ACCENT);
        let opts = [
            ("New Game / Continue", "Start or resume your run"),
            ("Load Save", "Reload last save file"),
            ("Quit", "Exit to desktop"),
        ];
        for (i, (o, sub)) in opts.iter().enumerate() {
            let y = my + 46.0 + i as f32 * 40.0;
            draw_select_row(mx + 12.0, y - 16.0, mw - 24.0, 36.0, i == self.title_idx);
            draw_text(
                o,
                mx + 30.0,
                y + 2.0,
                22.0,
                if i == self.title_idx { WHITE } else { C_TEXT },
            );
            draw_text(sub, mx + 30.0, y + 18.0, 13.0, C_TEXT_DIM);
        }

        draw_text(
            "Fan-made · Sprites via PokeAPI · Not affiliated with Nintendo",
            40.0,
            h - 24.0,
            14.0,
            C_TEXT_DIM,
        );
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_starter(&mut self) {
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            self.starter_idx = self.starter_idx.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            self.starter_idx = (self.starter_idx + 1).min(2);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            let starters = starter_species();
            let (sid, _name) = starters[self.starter_idx];
            if let Some(p) = PokemonInstance::from_species_id(sid, 5) {
                self.save.party.clear();
                self.save.party.push(p);
                self.save.dex.mark_caught(sid);
                self.save.starter_chosen = true;
                // buddy Pikachu? keep simple — just starter
                self.px = 8.0 * TILE_PX;
                self.py = 10.0 * TILE_PX;
                self.target_px = self.px;
                self.target_py = self.py;
                self.screen = Screen::Overworld;
                self.show_toast("Adventure begins! Talk to signs (E), then find tall grass.");
                self.try_save();
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Title;
        }
    }

    fn draw_starter(&self) {
        let w = screen_width();
        let t = get_time() as f32;
        draw_gradient_bg(
            Color::new(0.08, 0.14, 0.12, 1.0),
            Color::new(0.12, 0.22, 0.18, 1.0),
            22,
        );
        draw_vignette();
        draw_title_text("Choose your starter", w / 2.0 - 170.0, 56.0, 34.0);
        draw_text(
            "← → to browse · Enter to begin your journey",
            w / 2.0 - 160.0,
            84.0,
            16.0,
            C_TEXT_DIM,
        );

        let starters = starter_species();
        let type_cols = [
            Color::from_rgba(72, 192, 72, 255),
            Color::from_rgba(240, 96, 48, 255),
            Color::from_rgba(64, 144, 240, 255),
        ];
        let types = ["Grass", "Fire", "Water"];
        let descs = [
            "Steady growth & status",
            "Strong offense",
            "Balanced defender",
        ];
        let card_w = 200.0;
        let gap = 28.0;
        let total = 3.0 * card_w + 2.0 * gap;
        let start_x = (w - total) / 2.0;
        for (i, (sid, name)) in starters.iter().enumerate() {
            let x = start_x + i as f32 * (card_w + gap);
            let y = 120.0;
            let sel = i == self.starter_idx;
            let lift = if sel {
                (t * 3.0).sin().abs() * 4.0
            } else {
                0.0
            };
            draw_panel(x, y - lift, card_w, 300.0, sel);
            if sel {
                draw_rectangle(x + 4.0, y + 4.0 - lift, card_w - 8.0, 4.0, C_ACCENT);
                // Soft glow ring under selection
                draw_ellipse(
                    x + card_w * 0.5,
                    y + 290.0 - lift,
                    70.0,
                    10.0,
                    0.0,
                    Color::new(C_ACCENT.r, C_ACCENT.g, C_ACCENT.b, 0.22),
                );
            }
            let bob = if sel { (t * 2.2).sin() * 3.0 } else { 0.0 };
            draw_pokemon_sprite(
                &self.sprites,
                *sid,
                x + 36.0,
                y + 24.0 + bob - lift,
                128.0,
                false,
                0.0,
            );
            draw_text(name, x + 20.0, y + 180.0 - lift, 24.0, C_TEXT);
            draw_rectangle(x + 20.0, y + 194.0 - lift, 70.0, 18.0, type_cols[i]);
            draw_text(types[i], x + 30.0, y + 208.0 - lift, 14.0, WHITE);
            draw_text(descs[i], x + 16.0, y + 236.0 - lift, 13.0, C_TEXT_DIM);
            if sel {
                draw_text("SELECT", x + 62.0, y + 275.0 - lift, 16.0, C_ACCENT);
            }
        }
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_overworld(&mut self) {
        let dt = get_frame_time();
        self.walk_cd = (self.walk_cd - dt).max(0.0);
        self.toast_timer = (self.toast_timer - dt).max(0.0);
        self.autosave_cd = (self.autosave_cd - dt).max(0.0);
        self.tick_movement(dt);
        self.tick_grass_flecks(dt);

        // Smooth camera follow
        let (tx, ty) = self.desired_camera();
        if !self.cam_ready {
            self.cam_x = tx;
            self.cam_y = ty;
            self.cam_ready = true;
        } else {
            let k = (1.0 - (-12.0 * dt).exp()).clamp(0.0, 1.0); // framerate-independent ease
            self.cam_x += (tx - self.cam_x) * k;
            self.cam_y += (ty - self.cam_y) * k;
        }

        if is_key_pressed(KeyCode::Escape) {
            self.pause_idx = 0;
            self.screen = Screen::Pause;
            return;
        }
        if is_key_pressed(KeyCode::P) {
            self.party_idx = 0;
            self.party_from_pause = false;
            self.screen = Screen::Party;
            return;
        }
        if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::H) {
            self.try_interact();
        }

        // Hold-to-walk: try_move buffers while mid-slide
        let up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
        let down = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
        let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);

        if up {
            self.try_move(0, -1);
        } else if down {
            self.try_move(0, 1);
        } else if left {
            self.try_move(-1, 0);
        } else if right {
            self.try_move(1, 0);
        } else if !self.moving {
            self.pending_step = None;
        }
    }

    fn desired_camera(&self) -> (f32, f32) {
        let map_px_w = MAP_W as f32 * TILE_PX;
        let map_px_h = MAP_H as f32 * TILE_PX;
        let mut cam_x = self.px - screen_width() / 2.0 + TILE_PX / 2.0;
        let mut cam_y = self.py - screen_height() / 2.0 + TILE_PX / 2.0;
        cam_x = cam_x.clamp(0.0, (map_px_w - screen_width()).max(0.0));
        cam_y = cam_y.clamp(0.0, (map_px_h - screen_height()).max(0.0));
        (cam_x, cam_y)
    }

    fn draw_overworld(&self) {
        clear_background(Color::from_rgba(52, 110, 62, 255));
        let (want_x, want_y) = self.desired_camera();
        // Use smoothed camera if initialized; fall back to target on first frame
        let (cam_x, cam_y) = if self.cam_ready {
            (self.cam_x, self.cam_y)
        } else {
            (want_x, want_y)
        };
        draw_map(&self.map, cam_x, cam_y);
        draw_world_props(cam_x, cam_y);
        // Follower: first conscious party member (matches battle lead)
        if let Some(sid) = self.save.follower_species_id() {
            draw_follower(
                &self.sprites,
                sid,
                self.px,
                self.py,
                self.facing,
                cam_x,
                cam_y,
                self.walk_phase,
            );
        }
        draw_player(self.px, self.py, cam_x, cam_y, self.facing, self.walk_phase);

        // Grass step flecks (world-space)
        for f in &self.grass_flecks {
            let a = (f.life / 0.55).clamp(0.0, 1.0);
            draw_rectangle(
                f.x - cam_x,
                f.y - cam_y,
                3.0,
                5.0,
                Color::new(0.18, 0.55, 0.22, a * 0.85),
            );
        }

        // Center label plaque
        let cx = 5.5 * TILE_PX - cam_x;
        let cy = 14.2 * TILE_PX - cam_y;
        draw_panel(cx, cy, 92.0, 20.0, true);
        draw_text("POKé CENTER", cx + 8.0, cy + 14.0, 12.0, WHITE);

        let (gtx, gty) = self.grid_pos();
        draw_hud_bar(
            self.save.money,
            self.save.item_count(ItemKind::PokeBall),
            self.save.item_count(ItemKind::Potion),
            self.save.party.len(),
            area_name_for_tile(gtx, gty),
        );

        // Party HP orbs (quick glance)
        let orb_y = 42.0;
        for (i, p) in self.save.party.iter().enumerate().take(6) {
            let ox = 14.0 + i as f32 * 22.0;
            let ratio = if p.max_hp > 0 {
                p.current_hp as f32 / p.max_hp as f32
            } else {
                0.0
            };
            let col = if p.is_fainted() {
                Color::from_rgba(60, 60, 70, 255)
            } else {
                super::theme::hp_color(ratio)
            };
            draw_circle(ox, orb_y, 8.0, Color::from_rgba(20, 24, 36, 200));
            draw_circle(ox, orb_y, 6.5, col);
            if i == 0 && !p.is_fainted() {
                draw_circle_lines(ox, orb_y, 8.5, 1.5, C_ACCENT);
            }
        }

        // Subtle day/night wash
        let tod = ((get_time() * 0.035) as f32).sin() * 0.5 + 0.5;
        if tod > 0.58 {
            let night = ((tod - 0.58) / 0.42).clamp(0.0, 1.0);
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.04, 0.05, 0.18, night * 0.2),
            );
        }

        // Bottom help bar
        let bh = 30.0;
        draw_rectangle(
            0.0,
            screen_height() - bh,
            screen_width(),
            bh,
            Color::new(0.05, 0.07, 0.12, 0.88),
        );
        draw_rectangle(0.0, screen_height() - bh, screen_width(), 2.0, C_ACCENT);
        // Mart plaque
        let mx = 11.2 * TILE_PX - cam_x;
        let my_l = 14.2 * TILE_PX - cam_y;
        draw_panel(mx, my_l, 72.0, 20.0, false);
        draw_text("MART", mx + 16.0, my_l + 14.0, 12.0, WHITE);

        draw_text(
            "E/H talk/heal/shop  ·  Tall grass = battles  ·  Face signs/NPCs  ·  P party  ·  Esc pause/dex",
            14.0,
            screen_height() - 10.0,
            13.0,
            C_TEXT_DIM,
        );
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_party(&mut self) {
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            // Esc or P: return to Pause if Party was opened from Pause; else Overworld.
            if self.party_from_pause {
                self.party_from_pause = false;
                self.screen = Screen::Pause;
            } else {
                self.screen = Screen::Overworld;
            }
            return;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.party_idx = self.party_idx.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            if !self.save.party.is_empty() {
                self.party_idx = (self.party_idx + 1).min(self.save.party.len() - 1);
            }
        }
        // Use potion out of battle
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::U) {
            let idx = self.party_idx;
            if idx < self.save.party.len() && self.save.item_count(ItemKind::Potion) > 0 {
                let can_heal = {
                    let p = &self.save.party[idx];
                    p.current_hp < p.max_hp && !p.is_fainted()
                };
                if can_heal {
                    self.save.consume_item(ItemKind::Potion);
                    let heal = ItemKind::Potion.heal_amount();
                    let p = &mut self.save.party[idx];
                    p.current_hp = (p.current_hp + heal).min(p.max_hp);
                    self.show_toast("Used Potion!");
                }
            }
        }
        // Swap with lead ([1] or [L]) for strategic order out of battle
        if is_key_pressed(KeyCode::Key1) || is_key_pressed(KeyCode::L) {
            let idx = self.party_idx;
            if idx > 0 && idx < self.save.party.len() {
                self.save.party.swap(0, idx);
                self.party_idx = 0;
                self.show_toast("Sent to lead slot!");
            }
        }
    }

    fn draw_party(&self) {
        let w = screen_width();
        let h = screen_height();
        draw_gradient_bg(
            Color::new(0.07, 0.09, 0.16, 1.0),
            Color::new(0.10, 0.14, 0.24, 1.0),
            20,
        );
        draw_vignette();
        draw_title_text("Party", 40.0, 48.0, 34.0);
        draw_text(
            "↑↓ select  ·  Enter/U potion  ·  1/L set lead  ·  Esc/P back",
            40.0,
            72.0,
            15.0,
            C_TEXT_DIM,
        );

        if self.save.party.is_empty() {
            draw_panel(40.0, 100.0, w - 80.0, 80.0, false);
            draw_text("No Pokemon in your party yet.", 60.0, 148.0, 22.0, C_TEXT);
            return;
        }

        for (i, p) in self.save.party.iter().enumerate() {
            let y = 92.0 + i as f32 * 82.0;
            let sel = i == self.party_idx;
            draw_panel(30.0, y, w - 60.0, 74.0, sel);
            if i == 0 {
                draw_text("LEAD", w - 100.0, y + 22.0, 13.0, C_ACCENT);
            }
            draw_pokemon_sprite(&self.sprites, p.species_id, 44.0, y + 6.0, 60.0, false, 0.0);

            let status = if p.is_fainted() { "  [FNT]" } else { "" };
            draw_text(
                &format!("{}  Lv{}{}", p.display_name(), p.level, status),
                120.0,
                y + 26.0,
                22.0,
                C_TEXT,
            );

            // Mini HP bar
            let ratio = if p.max_hp > 0 {
                p.current_hp as f32 / p.max_hp as f32
            } else {
                0.0
            };
            let bx = 120.0;
            let by = y + 38.0;
            let bw = 260.0;
            draw_rectangle(bx, by, bw, 12.0, Color::new(0.1, 0.1, 0.12, 0.9));
            draw_rectangle(
                bx + 1.0,
                by + 1.0,
                (bw - 2.0) * ratio,
                10.0,
                super::theme::hp_color(ratio),
            );
            draw_rectangle(
                bx + 1.0,
                by + 1.0,
                (bw - 2.0) * ratio,
                4.0,
                Color::new(1.0, 1.0, 1.0, 0.18),
            );
            draw_text(
                &format!("{}/{}", p.current_hp.max(0), p.max_hp),
                bx + bw + 10.0,
                by + 11.0,
                14.0,
                C_TEXT_DIM,
            );

            // XP bar (party overview)
            let xr = Self::xp_progress(p);
            draw_rectangle(bx, by + 16.0, bw, 5.0, Color::from_rgba(30, 30, 48, 255));
            draw_rectangle(
                bx,
                by + 16.0,
                (bw * xr).max(1.0),
                5.0,
                Color::from_rgba(72, 148, 255, 255),
            );

            let mut tx = 120.0;
            for t in p.types.iter().take(2) {
                let (r, g, b) = t.rgb();
                let tw = measure_text(t.display_name(), None, 12, 1.0).width + 10.0;
                draw_rectangle(tx, y + 60.0, tw, 14.0, Color::from_rgba(r, g, b, 220));
                draw_text(t.display_name(), tx + 5.0, y + 71.0, 12.0, WHITE);
                tx += tw + 4.0;
            }
        }

        draw_panel(30.0, h - 48.0, 320.0, 32.0, false);
        draw_text(
            &format!(
                "Balls {}  ·  Potions {}  ·  Caught {}",
                self.save.item_count(ItemKind::PokeBall),
                self.save.item_count(ItemKind::Potion),
                self.save.pokemon_caught
            ),
            44.0,
            h - 26.0,
            15.0,
            C_TEXT_DIM,
        );
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_pause(&mut self) {
        let dt = get_frame_time();
        self.toast_timer = (self.toast_timer - dt).max(0.0);
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Overworld;
            return;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.pause_idx = self.pause_idx.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.pause_idx = (self.pause_idx + 1).min(4);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match self.pause_idx {
                0 => {
                    self.try_save();
                    self.screen = Screen::Overworld;
                }
                1 => {
                    self.party_idx = 0;
                    self.party_from_pause = true;
                    self.screen = Screen::Party;
                }
                2 => {
                    self.dex_sel = 0;
                    self.dex_scroll = 0;
                    self.screen = Screen::Pokedex;
                }
                3 => {
                    self.screen = Screen::Title;
                }
                _ => std::process::exit(0),
            }
        }
    }

    fn update_dialogue(&mut self) {
        let dt = get_frame_time();
        self.toast_timer = (self.toast_timer - dt).max(0.0);
        let advance = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::E);
        let close_now = is_key_pressed(KeyCode::Escape);
        if close_now {
            self.dialogue_prop = None;
            self.dialogue_line = 0;
            self.screen = Screen::Overworld;
            return;
        }
        if advance {
            if let Some(prop) = self.dialogue_prop {
                if self.dialogue_line + 1 < prop.lines.len() {
                    self.dialogue_line += 1;
                } else {
                    self.dialogue_prop = None;
                    self.dialogue_line = 0;
                    self.screen = Screen::Overworld;
                }
            } else {
                self.screen = Screen::Overworld;
            }
        }
    }

    fn draw_dialogue(&self) {
        self.draw_overworld();
        if let Some(prop) = self.dialogue_prop {
            draw_dialogue_box(prop, self.dialogue_line);
        }
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_pokedex(&mut self) {
        let dt = get_frame_time();
        self.toast_timer = (self.toast_timer - dt).max(0.0);
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
            self.screen = Screen::Pause;
            return;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.dex_sel = self.dex_sel.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.dex_sel = (self.dex_sel + 1).min((DEX_SIZE as usize).saturating_sub(1));
        }
        // Keep selection in view (8 rows visible)
        if self.dex_sel < self.dex_scroll {
            self.dex_scroll = self.dex_sel;
        }
        if self.dex_sel >= self.dex_scroll + 8 {
            self.dex_scroll = self.dex_sel - 7;
        }
    }

    fn draw_pokedex(&self) {
        let w = screen_width();
        let h = screen_height();
        draw_gradient_bg(
            Color::new(0.08, 0.10, 0.18, 1.0),
            Color::new(0.12, 0.16, 0.28, 1.0),
            20,
        );
        draw_vignette();
        let seen_n = self.save.dex.seen_count();
        let caught_n = self.save.dex.caught_count();
        let dex_max = DEX_SIZE;
        draw_title_text("Pokédex", 40.0, 48.0, 34.0);
        draw_text(
            &format!("Seen {seen_n}/{dex_max}  ·  Caught {caught_n}/{dex_max}  ·  Esc back"),
            40.0,
            72.0,
            15.0,
            C_TEXT_DIM,
        );

        let list_x = 40.0;
        let list_y = 96.0;
        let list_w = w * 0.48;
        draw_panel(list_x, list_y, list_w, h - list_y - 30.0, false);

        for row in 0..8 {
            let id = self.dex_scroll + row + 1;
            if id > DEX_SIZE as usize {
                break;
            }
            let sid = id as u16;
            let y = list_y + 16.0 + row as f32 * 52.0;
            let sel = self.dex_sel + 1 == id;
            draw_select_row(list_x + 8.0, y, list_w - 16.0, 46.0, sel);
            let seen = self.save.dex.has_seen(sid);
            let caught = self.save.dex.has_caught(sid);
            let name = if seen {
                species_by_id(sid)
                    .map(|sp| sp.name.clone())
                    .unwrap_or_else(|| format!("#{sid}"))
            } else {
                "???".into()
            };
            let mark = if caught {
                "●"
            } else if seen {
                "○"
            } else {
                "·"
            };
            draw_text(
                &format!("{mark}  #{sid:03}  {name}"),
                list_x + 20.0,
                y + 28.0,
                18.0,
                if seen { C_TEXT } else { C_TEXT_DIM },
            );
        }

        // Detail pane
        let dx = list_x + list_w + 24.0;
        let dw = w - dx - 40.0;
        draw_panel(dx, list_y, dw, h - list_y - 30.0, true);
        let sid = (self.dex_sel + 1) as u16;
        let seen = self.save.dex.has_seen(sid);
        let caught = self.save.dex.has_caught(sid);
        if seen {
            draw_pokemon_sprite(
                &self.sprites,
                sid,
                dx + dw * 0.5 - 64.0,
                list_y + 40.0,
                128.0,
                false,
                0.0,
            );
            if let Some(sp) = species_by_id(sid) {
                draw_text(&sp.name, dx + 24.0, list_y + 190.0, 26.0, C_TEXT);
                let mut tx = dx + 24.0;
                for t in sp.types.iter().take(2) {
                    let (r, g, b) = t.rgb();
                    let tw = measure_text(t.display_name(), None, 14, 1.0).width + 12.0;
                    draw_rectangle(tx, list_y + 204.0, tw, 18.0, Color::from_rgba(r, g, b, 220));
                    draw_text(t.display_name(), tx + 6.0, list_y + 218.0, 14.0, WHITE);
                    tx += tw + 6.0;
                }
                draw_text(
                    &format!(
                        "HP {}  Atk {}  Def {}  Spd {}",
                        sp.base_stats.hp,
                        sp.base_stats.attack,
                        sp.base_stats.defense,
                        sp.base_stats.speed
                    ),
                    dx + 24.0,
                    list_y + 250.0,
                    15.0,
                    C_TEXT_DIM,
                );
                draw_text(
                    if caught {
                        "Status: Caught ✓"
                    } else {
                        "Status: Seen"
                    },
                    dx + 24.0,
                    list_y + 278.0,
                    16.0,
                    if caught { C_ACCENT } else { C_TEXT },
                );
            }
        } else {
            draw_text(
                "???",
                dx + dw * 0.5 - 24.0,
                list_y + 120.0,
                42.0,
                C_TEXT_DIM,
            );
            draw_text(
                "Not seen yet — explore tall grass!",
                dx + 24.0,
                list_y + 200.0,
                16.0,
                C_TEXT_DIM,
            );
        }
    }

    fn draw_pause(&self) {
        self.draw_overworld();
        let w = screen_width();
        let h = screen_height();
        draw_modal_dim(0.58);

        let mw = 320.0;
        let mh = 350.0;
        let mx = w / 2.0 - mw / 2.0;
        let my = h / 2.0 - mh / 2.0;
        draw_panel(mx, my, mw, mh, true);
        draw_rectangle(mx + 10.0, my + 10.0, mw - 20.0, 4.0, C_ACCENT);
        draw_title_text("Paused", mx + 100.0, my + 44.0, 30.0);

        let opts = [
            ("Save Game", "Write progress to disk"),
            ("Party", "View & heal with potions"),
            ("Pokédex", "Seen & caught catalogue"),
            ("Title Screen", "Return to main menu"),
            ("Quit", "Close the game"),
        ];
        for (i, (o, sub)) in opts.iter().enumerate() {
            let y = my + 70.0 + i as f32 * 42.0;
            draw_select_row(mx + 14.0, y - 14.0, mw - 28.0, 38.0, i == self.pause_idx);
            draw_text(o, mx + 32.0, y + 4.0, 20.0, C_TEXT);
            draw_text(sub, mx + 32.0, y + 20.0, 12.0, C_TEXT_DIM);
        }
        let lead_lv = self
            .save
            .party
            .first()
            .map(|p| format!("Lead Lv{}", p.level))
            .unwrap_or_else(|| "No party".into());
        draw_text(
            &format!(
                "Caught {}  ·  Wins {}  ·  {}",
                self.save.pokemon_caught, self.save.battles_won, lead_lv
            ),
            mx + 24.0,
            my + mh - 18.0,
            13.0,
            C_TEXT_DIM,
        );
        draw_toast(&self.toast, self.toast_timer);
    }

    fn update_battle(&mut self) {
        let dt = get_frame_time();
        self.battle_flash = (self.battle_flash - dt).max(0.0);
        self.toast_timer = (self.toast_timer - dt).max(0.0);

        let Some(b) = &mut self.battle else {
            self.screen = Screen::Overworld;
            return;
        };
        b.shake_foe = (b.shake_foe - dt * 40.0).max(0.0);
        b.shake_player = (b.shake_player - dt * 40.0).max(0.0);
        b.hit_flash_foe = (b.hit_flash_foe - dt).max(0.0);
        b.hit_flash_player = (b.hit_flash_player - dt).max(0.0);
        // Smooth HP drain/heal in the UI
        let k = (1.0 - (-8.0 * dt).exp()).clamp(0.0, 1.0);
        let tgt_f = b.foe.current_hp.max(0) as f32;
        let tgt_p = b.player.current_hp.max(0) as f32;
        b.disp_hp_foe += (tgt_f - b.disp_hp_foe) * k;
        b.disp_hp_player += (tgt_p - b.disp_hp_player) * k;
        if (b.disp_hp_foe - tgt_f).abs() < 0.35 {
            b.disp_hp_foe = tgt_f;
        }
        if (b.disp_hp_player - tgt_p).abs() < 0.35 {
            b.disp_hp_player = tgt_p;
        }
        for f in &mut b.floats {
            f.tick(dt);
        }
        b.floats.retain(|f| f.alive());

        // Intro lockout — no input until sprites "settle"
        if b.intro_t > 0.0 {
            b.intro_t = (b.intro_t - dt).max(0.0);
            return;
        }

        match b.phase {
            BattlePhase::Won => {
                b.wait_timer -= dt;
                if b.wait_timer <= 0.0 {
                    let foe = b.foe.clone();
                    let pidx = b.player_idx;
                    if pidx < self.save.party.len() {
                        self.save.party[pidx] = b.player.clone();
                    }
                    self.save.battles_won += 1;
                    // Slightly better rewards for tougher foes
                    self.save.money += 25 + foe.level as u32 * 8;
                    self.apply_xp_to_player(&foe);
                    self.quiet_autosave();
                    self.end_battle_to_overworld();
                }
            }
            BattlePhase::Lost | BattlePhase::Caught | BattlePhase::Fled => {
                b.wait_timer -= dt;
                if b.wait_timer <= 0.0 {
                    if matches!(b.phase, BattlePhase::Caught) {
                        self.quiet_autosave();
                    }
                    self.end_battle_to_overworld();
                }
            }
            BattlePhase::FoeTurn => {
                b.wait_timer -= dt;
                if b.wait_timer <= 0.0 {
                    self.foe_attack();
                }
            }
            BattlePhase::BagMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    if let Some(b) = &mut self.battle {
                        b.phase = BattlePhase::PlayerPick;
                    }
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if let Some(b) = &mut self.battle {
                        b.bag_idx = b.bag_idx.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if let Some(b) = &mut self.battle {
                        b.bag_idx = (b.bag_idx + 1).min(2);
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let idx = self.battle.as_ref().map(|b| b.bag_idx).unwrap_or(2);
                    match idx {
                        0 => self.try_catch(),
                        1 => self.try_potion_in_battle(),
                        _ => {
                            if let Some(b) = &mut self.battle {
                                b.phase = BattlePhase::PlayerPick;
                            }
                        }
                    }
                }
            }
            BattlePhase::SwitchMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    if let Some(b) = &mut self.battle {
                        b.phase = BattlePhase::PlayerPick;
                    }
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if let Some(b) = &mut self.battle {
                        b.switch_idx = b.switch_idx.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    let max = self.save.party.len().saturating_sub(1);
                    if let Some(b) = &mut self.battle {
                        b.switch_idx = (b.switch_idx + 1).min(max);
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let idx = self.battle.as_ref().map(|b| b.switch_idx).unwrap_or(0);
                    self.try_switch_in_battle(idx);
                }
            }
            BattlePhase::PlayerMoves => {
                if is_key_pressed(KeyCode::Escape) {
                    if let Some(b) = &mut self.battle {
                        b.phase = BattlePhase::PlayerPick;
                    }
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if let Some(b) = &mut self.battle {
                        b.move_idx = b.move_idx.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if let Some(b) = &mut self.battle {
                        let max = b.player.moves.len().saturating_sub(1);
                        b.move_idx = (b.move_idx + 1).min(max);
                    }
                }
                // Number keys 1-4 quick-select moves
                for (k, idx) in [
                    (KeyCode::Key1, 0usize),
                    (KeyCode::Key2, 1),
                    (KeyCode::Key3, 2),
                    (KeyCode::Key4, 3),
                ] {
                    if is_key_pressed(k) {
                        let n = self
                            .battle
                            .as_ref()
                            .map(|b| b.player.moves.len())
                            .unwrap_or(0);
                        if idx < n {
                            self.player_attack(idx);
                            break;
                        }
                    }
                }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    let mi = self.battle.as_ref().map(|b| b.move_idx).unwrap_or(0);
                    self.player_attack(mi);
                }
            }
            BattlePhase::PlayerPick => {
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if let Some(b) = &mut self.battle {
                        b.menu_idx = b.menu_idx.saturating_sub(1);
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if let Some(b) = &mut self.battle {
                        b.menu_idx = (b.menu_idx + 1).min(3);
                    }
                }
                // 1-4 quick-pick command (matches on-screen key hints)
                for (k, idx) in [
                    (KeyCode::Key1, 0usize),
                    (KeyCode::Key2, 1),
                    (KeyCode::Key3, 2),
                    (KeyCode::Key4, 3),
                ] {
                    if is_key_pressed(k) {
                        if let Some(b) = &mut self.battle {
                            b.menu_idx = idx;
                        }
                        // Fall through to confirm below by setting a one-shot confirm
                        // (handled by Enter path via synthetic select)
                    }
                }
                let num_confirm = is_key_pressed(KeyCode::Key1)
                    || is_key_pressed(KeyCode::Key2)
                    || is_key_pressed(KeyCode::Key3)
                    || is_key_pressed(KeyCode::Key4);
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || num_confirm {
                    let idx = self.battle.as_ref().map(|b| b.menu_idx).unwrap_or(0);
                    match idx {
                        0 => {
                            if let Some(b) = &mut self.battle {
                                b.phase = BattlePhase::PlayerMoves;
                                b.move_idx = 0;
                            }
                        }
                        1 => {
                            if let Some(b) = &mut self.battle {
                                b.phase = BattlePhase::BagMenu;
                                b.bag_idx = 0;
                            }
                        }
                        2 => {
                            if let Some(b) = &mut self.battle {
                                b.phase = BattlePhase::SwitchMenu;
                                b.switch_idx = 0;
                            }
                        }
                        3 => {
                            // run — easier in wild
                            let ok = self
                                .battle
                                .as_ref()
                                .map(|b| b.is_wild && (::rand::random::<u8>() % 100) < 70)
                                .unwrap_or(false);
                            if let Some(b) = &mut self.battle {
                                if ok {
                                    b.log.push("Got away safely!".into());
                                    b.phase = BattlePhase::Fled;
                                    b.wait_timer = 0.8;
                                } else {
                                    b.log.push("Can't escape!".into());
                                    b.phase = BattlePhase::FoeTurn;
                                    b.wait_timer = 0.4;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw_battle(&self) {
        let Some(b) = &self.battle else { return };
        draw_battle_backdrop();

        let w = screen_width();
        let h = screen_height();
        let shake_sign = if (get_time() * 30.0) as i32 % 2 == 0 {
            1.0
        } else {
            -1.0
        };

        // Intro: slide sprites in from sides
        let intro = b.intro_t;
        let slide = if intro > 0.0 {
            (intro / 0.85) * 220.0
        } else {
            0.0
        };
        // Intro white flash stripes (classic battle transition residue)
        if intro > 0.35 {
            let fa = ((intro - 0.35) / 0.5).clamp(0.0, 1.0) * 0.55;
            for i in 0..8 {
                let yy = (i as f32 / 8.0) * h;
                if i % 2 == 0 {
                    draw_rectangle(0.0, yy, w, h / 8.0 + 1.0, Color::new(1.0, 1.0, 1.0, fa));
                }
            }
        }

        // Hit flash tint layers behind sprites
        if b.hit_flash_foe > 0.0 {
            draw_rectangle(
                w * 0.52,
                h * 0.08,
                200.0,
                180.0,
                Color::new(1.0, 0.2, 0.2, b.hit_flash_foe * 1.2),
            );
        }
        if b.hit_flash_player > 0.0 {
            draw_rectangle(
                w * 0.06,
                h * 0.34,
                220.0,
                190.0,
                Color::new(1.0, 0.2, 0.2, b.hit_flash_player * 1.2),
            );
        }

        // Foe (front sprite, top-right)
        draw_pokemon_sprite(
            &self.sprites,
            b.foe.species_id,
            w * 0.56 + slide,
            h * 0.10,
            168.0,
            false,
            b.shake_foe * shake_sign,
        );
        let foe_show_hp = b.disp_hp_foe.round() as i64;
        draw_status_box(
            24.0,
            28.0,
            280.0,
            b.foe.display_name(),
            b.foe.level,
            foe_show_hp,
            b.foe.max_hp,
            &b.foe.types,
            None,
        );
        if b.is_wild {
            draw_text("WILD", 250.0, 50.0, 14.0, C_ACCENT);
        }

        // Player (back sprite, bottom-left)
        draw_pokemon_sprite(
            &self.sprites,
            b.player.species_id,
            w * 0.10 - slide,
            h * 0.36,
            176.0,
            true,
            b.shake_player * shake_sign,
        );
        let xp_r = Self::xp_progress(&b.player);
        let pl_show_hp = b.disp_hp_player.round() as i64;
        draw_status_box(
            w * 0.52,
            h * 0.46,
            300.0,
            b.player.display_name(),
            b.player.level,
            pl_show_hp,
            b.player.max_hp,
            &b.player.types,
            Some(xp_r),
        );

        draw_battle_log(&b.log);
        draw_float_texts(&b.floats);

        if intro <= 0.0 {
            match b.phase {
                BattlePhase::PlayerPick => {
                    draw_battle_menu(b.menu_idx, false, 0, &b.player, b.is_wild, &b.foe.types);
                }
                BattlePhase::PlayerMoves => {
                    draw_battle_menu(
                        b.menu_idx,
                        true,
                        b.move_idx,
                        &b.player,
                        b.is_wild,
                        &b.foe.types,
                    );
                }
                BattlePhase::BagMenu => {
                    let catch_s;
                    let catch_hint = if b.is_wild {
                        catch_s = format!("Catch chance ~{}%", Self::estimate_catch_pct(&b.foe));
                        Some(catch_s.as_str())
                    } else {
                        None
                    };
                    draw_bag_menu(
                        b.bag_idx,
                        self.save.item_count(ItemKind::PokeBall),
                        self.save.item_count(ItemKind::Potion),
                        catch_hint,
                    );
                }
                BattlePhase::SwitchMenu => {
                    draw_switch_menu(b.switch_idx, &self.save.party, b.player_idx);
                }
                BattlePhase::Won => {
                    draw_overlay_banner("Victory!", C_ACCENT);
                    draw_text(
                        "XP & money awarded…",
                        w / 2.0 - 80.0,
                        h * 0.38 + 78.0,
                        16.0,
                        C_TEXT_DIM,
                    );
                }
                BattlePhase::Lost => {
                    draw_overlay_banner("Defeated...", Color::from_rgba(255, 90, 90, 255));
                    draw_text(
                        "You'll wake up at the Center.",
                        w / 2.0 - 110.0,
                        h * 0.38 + 78.0,
                        16.0,
                        C_TEXT_DIM,
                    );
                }
                BattlePhase::Caught => {
                    draw_overlay_banner("Caught!", Color::from_rgba(90, 220, 120, 255));
                }
                BattlePhase::Fled => {
                    draw_overlay_banner("Got away!", C_TEXT);
                }
                _ => {}
            }
        } else {
            let pulse = ((get_time() * 4.0) as f32).sin() * 0.15 + 0.85;
            draw_text(
                "A wild Pokémon appeared!",
                w / 2.0 - 110.0,
                h * 0.55,
                22.0,
                Color::new(1.0, 1.0, 1.0, pulse),
            );
        }

        // Encounter flash
        if self.battle_flash > 0.0 {
            let a = (self.battle_flash / 0.55).clamp(0.0, 1.0);
            draw_rectangle(0.0, 0.0, w, h, Color::new(1.0, 1.0, 1.0, a * 0.75));
        }
    }

    pub fn update(&mut self) {
        match self.screen {
            Screen::Title => self.update_title(),
            Screen::Starter => self.update_starter(),
            Screen::Overworld => self.update_overworld(),
            Screen::Battle => self.update_battle(),
            Screen::Party => self.update_party(),
            Screen::Pause => self.update_pause(),
            Screen::Shop => self.update_shop(),
            Screen::Dialogue => self.update_dialogue(),
            Screen::Pokedex => self.update_pokedex(),
        }
    }

    pub fn draw(&self) {
        match self.screen {
            Screen::Title => self.draw_title(),
            Screen::Starter => self.draw_starter(),
            Screen::Overworld => self.draw_overworld(),
            Screen::Battle => self.draw_battle(),
            Screen::Party => self.draw_party(),
            Screen::Pause => self.draw_pause(),
            Screen::Shop => self.draw_shop(),
            Screen::Dialogue => self.draw_dialogue(),
            Screen::Pokedex => self.draw_pokedex(),
        }
        // Always show toast on top for non-overworld too
        if !matches!(
            self.screen,
            Screen::Overworld | Screen::Shop | Screen::Pause | Screen::Dialogue
        ) {
            draw_toast(&self.toast, self.toast_timer);
        }
    }
}

pub async fn run_game() {
    let mut game = Game::new().await;
    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
