/* MIT sn99 — TUI application state */
mod audio;
mod input;
mod ui;

pub use audio::{AudioManager, MusicTrack};
pub use input::handle_key;
pub use ui::draw;

use rand::Rng;
use ratatui::widgets::ListState;

use ratatui::text::Line;

use pokemon_text_game::ascii::{
    battle_frame, color_sprite_for_species, flip_horizontal, sprite_for_species, ColorSprite,
};
use pokemon_text_game::battle::{
    apply_status_residual, apply_weather_residual, elite_by_index, elite_lead_instance,
    execute_move_weather, exhibition_foe, gym_by_index, gym_lead_instance, rival_instance,
    roll_block, roll_critical_chance, route_by_id, route_trainer_instance, starter_party,
    trainers_on_route, water_on_route, wild_on_route,
};
use pokemon_text_game::data::{read_team_from_file, write_team_to_file, TEAM_PATH};
use pokemon_text_game::pokemon::species::species_by_id;
use pokemon_text_game::pokemon::{all_species, ElementType, Pokemon, PokemonInstance, PokemonsList};
use pokemon_text_game::save::{
    default_save_path, export_save_copy, read_save, roll_capture, write_save, ItemKind, SaveGame,
};
use pokemon_text_game::util::{clamp_index, move_selection, parse_i64, parse_moves};
use pokemon_text_game::world::{
    daily_challenge_for_day, default_gyms, default_routes, mart_catalog, species_flavor, Weather,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    PlayMode,
    SelectPokemon { slot: SelectSlot },
    Battle,
    BattleOver,
    PokedexMenu,
    PokedexList { action: PokedexAction },
    SpeciesDex,
    FormInput,
    Message,
    AdventureHub,
    PartyView,
    InventoryView,
    SettingsView,
    StarterSelect,
    Help,
    RouteSelect,
    GymSelect,
    Achievements,
    BoxStorage,
    TypeChart,
    Mart,
    EliteFour,
    NicknameInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectSlot { VsComputerPlayer, VsHumanP1, VsHumanP2 }

impl SelectSlot {
    pub fn title(self) -> &'static str {
        match self {
            Self::VsComputerPlayer => " Choose your Pokemon ",
            Self::VsHumanP1 => " Player 1 — choose Pokemon ",
            Self::VsHumanP2 => " Player 2 — choose Pokemon ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PokedexAction { Edit, Delete }
impl PokedexAction {
    pub fn title(self) -> &'static str {
        match self { Self::Edit => " Select to edit ", Self::Delete => " Select to delete " }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind { CreatePokemon, EditPokemon { index: usize } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField { Name, Moves, Health, Type }
impl FormField {
    pub const ALL: [Self; 4] = [Self::Name, Self::Moves, Self::Health, Self::Type];
    pub fn next(self) -> Self { let i = Self::ALL.iter().position(|&f| f == self).unwrap_or(0); Self::ALL[(i + 1) % 4] }
    pub fn prev(self) -> Self { let i = Self::ALL.iter().position(|&f| f == self).unwrap_or(0); Self::ALL[(i + 3) % 4] }
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name", Self::Moves => "Moves (space-separated)",
            Self::Health => "Health", Self::Type => "Type (number or name)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleMode { Classic, Advanced }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleKind { Wild, Trainer, Gym, Exhibition, Classic, Rival, Elite }

pub struct BattleState {
    pub mode: BattleMode,
    pub kind: BattleKind,
    pub player1: Pokemon,
    pub player2: Pokemon,
    pub player1_max_hp: i64,
    pub player2_max_hp: i64,
    pub inst_p1: Option<PokemonInstance>,
    pub inst_p2: Option<PokemonInstance>,
    pub vs_computer: bool,
    pub is_wild: bool,
    pub can_catch: bool,
    pub gym_index: Option<u8>,
    pub player1_turn: bool,
    pub move_cursor: usize,
    pub battle_menu: usize, // 0=fight 1=bag 2=switch 3=run
    pub in_move_pick: bool,
    pub log: Vec<String>,
    pub winner_text: Option<String>,
    pub sprite_lines: Vec<String>,
    /// Colored battlefield lines (preferred when non-empty).
    pub color_sprite_lines: Vec<Line<'static>>,
    pub catch_attempts: u8,
    pub weather: Weather,
    pub trainer_reward: u32,
    pub elite_index: Option<u8>,
}

impl BattleState {
    pub fn move_labels(&self) -> Vec<String> {
        match self.mode {
            BattleMode::Classic => {
                let m = if self.player1_turn { &self.player1.moves_name } else { &self.player2.moves_name };
                m.clone()
            }
            BattleMode::Advanced => {
                let inst = if self.player1_turn { &self.inst_p1 } else { &self.inst_p2 };
                inst.as_ref().map(|p| p.moves.iter().map(|m| {
                    let eff = if let Some(def) = if self.player1_turn { &self.inst_p2 } else { &self.inst_p1 } {
                        let mult = pokemon_text_game::pokemon::type_effectiveness(m.data.move_type, &def.types);
                        if mult >= 2.0 { " ★" } else if mult == 0.0 { " ✕" } else if mult <= 0.5 { " ▽" } else { "" }
                    } else { "" };
                    format!("{} [{}] {}/{}PP{}", m.data.name, m.data.move_type.display_name(), m.current_pp, m.data.pp, eff)
                }).collect()).unwrap_or_default()
            }
        }
    }
    fn attacker_name(&self) -> String {
        match self.mode {
            BattleMode::Classic => if self.player1_turn { self.player1.name.clone() } else { self.player2.name.clone() },
            BattleMode::Advanced => {
                let i = if self.player1_turn { &self.inst_p1 } else { &self.inst_p2 };
                i.as_ref().map(|p| p.display_name().to_string()).unwrap_or_else(|| "?".into())
            }
        }
    }
    fn defender_name(&self) -> String {
        match self.mode {
            BattleMode::Classic => if self.player1_turn { self.player2.name.clone() } else { self.player1.name.clone() },
            BattleMode::Advanced => {
                let i = if self.player1_turn { &self.inst_p2 } else { &self.inst_p1 };
                i.as_ref().map(|p| p.display_name().to_string()).unwrap_or_else(|| "?".into())
            }
        }
    }
    pub fn side_label(&self, is_p1: bool) -> &'static str {
        match (is_p1, self.kind) {
            (true, _) => "You",
            (false, BattleKind::Wild) => "Wild",
            (false, BattleKind::Gym) => "Gym",
            (false, BattleKind::Exhibition) => "Dex Foe",
            (false, BattleKind::Rival) => "Rival",
            (false, BattleKind::Elite) => "Elite",
            (false, _) => "Foe",
        }
    }
    pub fn turn_prompt(&self) -> &'static str {
        if !self.player1_turn && self.vs_computer { "Foe is attacking..." }
        else if self.in_move_pick { "Choose a move" }
        else { "Fight / Ball / Item / Switch / Run" }
    }
    pub fn refresh_sprites(&mut self, show: bool) {
        if !show {
            self.sprite_lines.clear();
            self.color_sprite_lines.clear();
            return;
        }
        let (p1_id, p1_t, p1_n, p2_id, p2_t, p2_n, shiny1, shiny2) = match self.mode {
            BattleMode::Classic => {
                let t1 = ElementType::from_legacy_id(self.player1.pokemon_type);
                let t2 = ElementType::from_legacy_id(self.player2.pokemon_type);
                let id1 = species_by_id_name(&self.player1.name);
                let id2 = species_by_id_name(&self.player2.name);
                (id1, t1, self.player1.name.clone(), id2, t2, self.player2.name.clone(), false, false)
            }
            BattleMode::Advanced => {
                let p1 = self.inst_p1.as_ref(); let p2 = self.inst_p2.as_ref();
                (p1.map(|p| p.species_id).unwrap_or(0), p1.map(|p| p.primary_type()).unwrap_or(ElementType::Normal),
                 p1.map(|p| p.display_name().to_string()).unwrap_or_default(),
                 p2.map(|p| p.species_id).unwrap_or(0), p2.map(|p| p.primary_type()).unwrap_or(ElementType::Normal),
                 p2.map(|p| p.display_name().to_string()).unwrap_or_default(),
                 p1.map(|p| p.shiny).unwrap_or(false), p2.map(|p| p.shiny).unwrap_or(false))
            }
        };
        // Colored sprites (primary)
        let mut c1 = color_sprite_for_species(p1_id, p1_t).flip_horizontal();
        let c2 = color_sprite_for_species(p2_id, p2_t);
        self.color_sprite_lines = ColorSprite::battle_lines(&c1, &c2, &p1_n, &p2_n, shiny1, shiny2);
        // Mono fallback
        let mut s1 = sprite_for_species(p1_id, p1_t);
        s1 = flip_horizontal(&s1);
        let s2 = sprite_for_species(p2_id, p2_t);
        self.sprite_lines = battle_frame(&s1, &s2, &p1_n, &p2_n);
    }
}

fn species_by_id_name(name: &str) -> u16 {
    pokemon_text_game::pokemon::species_by_name(name).map(|s| s.id).unwrap_or(0)
}

pub struct FormState {
    pub kind: FormKind, pub field: FormField,
    pub name: String, pub moves: String, pub health: String, pub pokemon_type: String,
    pub error: Option<String>,
}
impl FormState {
    fn create() -> Self { Self { kind: FormKind::CreatePokemon, field: FormField::Name, name: String::new(), moves: String::new(), health: "400".into(), pokemon_type: "1".into(), error: None } }
    fn edit(index: usize, p: &Pokemon) -> Self { Self { kind: FormKind::EditPokemon { index }, field: FormField::Name, name: p.name.clone(), moves: p.moves_name.join(" "), health: p.health.to_string(), pokemon_type: p.pokemon_type.to_string(), error: None } }
    pub fn field_buf_mut(&mut self) -> &mut String { match self.field { FormField::Name => &mut self.name, FormField::Moves => &mut self.moves, FormField::Health => &mut self.health, FormField::Type => &mut self.pokemon_type } }
    pub fn field_value(&self, field: FormField) -> &str { match field { FormField::Name => &self.name, FormField::Moves => &self.moves, FormField::Health => &self.health, FormField::Type => &self.pokemon_type } }
    fn set_error(&mut self, msg: impl Into<String>) { self.error = Some(msg.into()); }
}

pub struct App {
    pub screen: Screen,
    pub team: PokemonsList,
    pub save: SaveGame,
    pub list_state: ListState,
    pub menu_index: usize,
    pub status: String,
    pub message: String,
    pub message_return: Screen,
    pub form: Option<FormState>,
    pub battle: Option<BattleState>,
    pub pending_select: Option<SelectSlot>,
    pub selected_p1_index: Option<usize>,
    pub should_quit: bool,
    pub species_index: usize,
    pub show_sprites: bool,
    pub dex_filter: DexFilter,
    pub dex_search: String,
    pub nickname_buf: String,
    pub nickname_target: usize,
    pub session_ticks: u64,
    pub pending_audio_volume: Option<f32>,
    pub pending_audio_enabled: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DexFilter { All, Seen, Caught, ByType(ElementType) }

impl App {
    pub fn new() -> Self {
        let (team, load_error) = match read_team_from_file(TEAM_PATH) {
            Ok(t) => (t, None),
            Err(e) => (PokemonsList::default(), Some(format!("Could not load {TEAM_PATH}: {e}"))),
        };
        let save = read_save(default_save_path()).unwrap_or_else(|_| {
            let mut s = SaveGame::default();
            for leg in &team.pokeball { s.party.push(leg.to_instance()); s.dex.mark_caught(leg.to_instance().species_id); }
            s
        });
        let mut list_state = ListState::default();
        if !team.pokeball.is_empty() { list_state.select(Some(0)); }
        let show_sprites = save.settings.show_sprites;
        let mut app = Self {
            screen: Screen::MainMenu, team, save, list_state, menu_index: 0,
            status: String::new(), message: String::new(), message_return: Screen::MainMenu,
            form: None, battle: None, pending_select: None, selected_p1_index: None,
            should_quit: false, species_index: 0, show_sprites,
            dex_filter: DexFilter::All, dex_search: String::new(),
            nickname_buf: String::new(), nickname_target: 0,
            session_ticks: 0, pending_audio_volume: None, pending_audio_enabled: None,
        };
        if let Some(err) = load_error { app.show_message(err, Screen::MainMenu); }
        app
    }

    /// Called each frame (~80ms) for play-time tracking.
    pub fn tick(&mut self) {
        self.session_ticks += 1;
        if self.session_ticks % 12 == 0 {
            // ~1 second at 80ms poll
            self.save.add_play_time(1);
        }
        if self.session_ticks % 750 == 0 {
            // Auto-save every ~60s
            let _ = write_save(default_save_path(), &self.save);
        }
    }

    pub fn persist_save(&mut self) {
        self.save.settings.show_sprites = self.show_sprites;
        match write_save(default_save_path(), &self.save) {
            Ok(()) => self.status = format!("Game saved. Play time {}", self.save.play_time_display()),
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }

    pub fn export_save(&mut self) {
        match export_save_copy(&self.save) {
            Ok(p) => self.status = format!("Exported save to {}", p.display()),
            Err(e) => self.status = format!("Export failed: {e}"),
        }
    }

    fn save_team(&mut self) -> bool {
        match write_team_to_file(TEAM_PATH, &self.team) {
            Ok(()) => { self.status = "Team file saved.".into(); true }
            Err(e) => { self.status = format!("Save failed: {e}"); false }
        }
    }

    fn show_message(&mut self, msg: impl Into<String>, ret: Screen) {
        self.message = msg.into(); self.message_return = ret; self.screen = Screen::Message;
    }

    pub fn go_main_menu(&mut self) { self.menu_index = 0; self.screen = Screen::MainMenu; }
    pub fn go_play_mode(&mut self) { self.menu_index = 0; self.screen = Screen::PlayMode; }
    pub fn go_pokedex_menu(&mut self) { self.menu_index = 0; self.screen = Screen::PokedexMenu; }
    pub fn go_adventure(&mut self) {
        self.menu_index = 0;
        if self.save.party.is_empty() { self.screen = Screen::StarterSelect; } else { self.screen = Screen::AdventureHub; }
    }
    pub fn go_help(&mut self) { self.screen = Screen::Help; }
    pub fn go_settings(&mut self) { self.menu_index = 0; self.screen = Screen::SettingsView; }
    pub fn go_species_dex(&mut self) { self.species_index = 0; self.screen = Screen::SpeciesDex; }
    pub fn go_type_chart(&mut self) { self.screen = Screen::TypeChart; }
    pub fn go_achievements(&mut self) { self.screen = Screen::Achievements; }
    pub fn go_routes(&mut self) { self.menu_index = 0; self.screen = Screen::RouteSelect; }
    pub fn go_gyms(&mut self) { self.menu_index = 0; self.screen = Screen::GymSelect; }
    pub fn go_box(&mut self) { self.menu_index = 0; self.screen = Screen::BoxStorage; }
    pub fn go_mart(&mut self) { self.menu_index = 0; self.screen = Screen::Mart; }
    pub fn go_elite(&mut self) {
        if self.save.badge_count() < 8 {
            self.show_message("Defeat all 8 gyms before the Elite Four!", Screen::AdventureHub);
            return;
        }
        self.menu_index = 0;
        self.screen = Screen::EliteFour;
    }

    pub fn filtered_species_indices(&self) -> Vec<usize> {
        let all = all_species();
        let q = self.dex_search.to_lowercase();
        all.iter().enumerate().filter(|(_, s)| {
            if !q.is_empty() && !s.name.to_lowercase().contains(&q) && !s.id.to_string().contains(&q) {
                return false;
            }
            match self.dex_filter {
                DexFilter::All => true,
                DexFilter::Seen => self.save.dex.seen.contains(&s.id),
                DexFilter::Caught => self.save.dex.caught.contains(&s.id),
                DexFilter::ByType(t) => s.types.contains(&t),
            }
        }).map(|(i, _)| i).collect()
    }

    pub fn selected_index(&self) -> Option<usize> { self.list_state.selected() }
    fn ensure_list_selection(&mut self) {
        let len = self.team.pokeball.len();
        if len == 0 { self.list_state.select(None); }
        else { let i = self.list_state.selected().unwrap_or(0); self.list_state.select(Some(clamp_index(i, len))); }
    }
    pub fn move_list(&mut self, delta: isize) {
        let len = self.team.pokeball.len(); if len == 0 { return; }
        let cur = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(move_selection(cur, delta, len)));
    }
    pub fn move_menu(&mut self, delta: isize, items: usize) { self.menu_index = move_selection(self.menu_index, delta, items); }
    pub fn move_species(&mut self, delta: isize) {
        let idxs = self.filtered_species_indices();
        if idxs.is_empty() { return; }
        let pos = idxs.iter().position(|&i| i == self.species_index).unwrap_or(0);
        let np = move_selection(pos, delta, idxs.len());
        self.species_index = idxs[np];
        if let Some(s) = all_species().get(self.species_index) {
            self.save.dex.mark_seen(s.id);
        }
    }

    pub fn open_pokedex_list(&mut self, action: PokedexAction) {
        if self.team.pokeball.is_empty() { self.status = "No Pokemon.".into(); return; }
        self.ensure_list_selection(); self.screen = Screen::PokedexList { action };
    }
    pub fn start_form_create(&mut self) { self.form = Some(FormState::create()); self.screen = Screen::FormInput; }
    pub fn start_form_edit(&mut self, index: usize) {
        let p = &self.team.pokeball[index]; self.form = Some(FormState::edit(index, p)); self.screen = Screen::FormInput;
    }

    pub fn submit_form(&mut self) {
        let Some(form) = self.form.as_ref() else { return; };
        let name = form.name.trim().to_string();
        if name.is_empty() { if let Some(f) = self.form.as_mut() { f.set_error("Name cannot be empty."); } return; }
        let moves = parse_moves(&form.moves);
        if moves.is_empty() { if let Some(f) = self.form.as_mut() { f.set_error("Enter at least one move."); } return; }
        let health = match parse_i64(&form.health) {
            Ok(h) if h > 0 => h,
            Ok(_) => { if let Some(f) = self.form.as_mut() { f.set_error("Health must be positive."); } return; }
            Err(e) => { if let Some(f) = self.form.as_mut() { f.set_error(e); } return; }
        };
        let pokemon_type = if let Ok(t) = parse_i64(&form.pokemon_type) { t }
        else if let Some(et) = ElementType::from_str_loose(&form.pokemon_type) {
            match et { ElementType::Electric=>1, ElementType::Grass=>2, ElementType::Water=>3, ElementType::Fire=>4, ElementType::Psychic=>6, ElementType::Fighting=>7, ElementType::Ghost=>8, ElementType::Dragon=>9, _=>0 }
        } else { if let Some(f) = self.form.as_mut() { f.set_error("Bad type."); } return; };
        let kind = form.kind;
        match kind {
            FormKind::CreatePokemon => {
                self.team.pokeball.push(Pokemon::with_stats(name, moves, health, pokemon_type));
                self.save_team(); self.form = None; self.ensure_list_selection();
                self.show_message("Pokemon created!", Screen::PokedexMenu);
            }
            FormKind::EditPokemon { index } => {
                if let Some(p) = self.team.pokeball.get_mut(index) { p.edit_full(name, moves, health, pokemon_type); }
                self.save_team(); self.form = None; self.show_message("Updated!", Screen::PokedexMenu);
            }
        }
    }

    pub fn delete_selected(&mut self) {
        let Some(idx) = self.selected_index() else { return; };
        if idx >= self.team.pokeball.len() { return; }
        let name = self.team.pokeball[idx].name.clone();
        self.team.pokeball.remove(idx); self.save_team(); self.ensure_list_selection();
        self.show_message(format!("Deleted {name}."), Screen::PokedexMenu);
    }

    pub fn begin_pokemon_select(&mut self, slot: SelectSlot) {
        if self.team.pokeball.is_empty() { self.show_message("Create Pokemon first.", Screen::MainMenu); return; }
        self.pending_select = Some(slot); self.ensure_list_selection(); self.screen = Screen::SelectPokemon { slot };
    }

    pub fn confirm_pokemon_select(&mut self) {
        let Some(slot) = self.pending_select else { return; };
        let Some(idx) = self.selected_index() else { return; };
        if idx >= self.team.pokeball.len() { return; }
        let mut rng = rand::thread_rng();
        match slot {
            SelectSlot::VsComputerPlayer => {
                let player = self.team.pokeball[idx].clone();
                let cpu = self.team.pokeball[rng.gen_range(0..self.team.pokeball.len())].clone();
                self.start_battle_classic(player, cpu, true, rng.gen_bool(0.5));
            }
            SelectSlot::VsHumanP1 => { self.selected_p1_index = Some(idx); self.begin_pokemon_select(SelectSlot::VsHumanP2); }
            SelectSlot::VsHumanP2 => {
                let p1 = self.team.pokeball[self.selected_p1_index.unwrap_or(0)].clone();
                let p2 = self.team.pokeball[idx].clone();
                self.selected_p1_index = None;
                self.start_battle_classic(p1, p2, false, rng.gen_bool(0.5));
            }
        }
    }

    fn start_battle_classic(&mut self, p1: Pokemon, p2: Pokemon, vs_computer: bool, player1_first: bool) {
        let mut log = vec![format!("{} vs {}!", p1.name, p2.name)];
        let mut battle = BattleState {
            mode: BattleMode::Classic, kind: BattleKind::Classic,
            player1_max_hp: p1.health, player2_max_hp: p2.health,
            player1: p1, player2: p2, inst_p1: None, inst_p2: None,
            vs_computer, is_wild: false, can_catch: false, gym_index: None,
            player1_turn: player1_first, move_cursor: 0, battle_menu: 0, in_move_pick: true,
            log, winner_text: None, sprite_lines: Vec::new(), color_sprite_lines: Vec::new(),
            catch_attempts: 0, weather: Weather::Clear, trainer_reward: 0, elite_index: None,
        };
        battle.refresh_sprites(self.show_sprites);
        self.battle = Some(battle); self.pending_select = None; self.screen = Screen::Battle;
        if !player1_first && vs_computer { self.resolve_cpu_turn(); }
    }

    pub fn pick_starter(&mut self, choice: usize) {
        let ids = [1u16, 4, 7];
        let starter_line = ids[choice % 3];
        let party = starter_party(choice);
        let names: Vec<_> = party.iter().map(|p| p.display_name().to_string()).collect();
        self.save = SaveGame::new_game(&self.save.player_name, party);
        self.save.starter_line = starter_line;
        self.persist_save();
        self.show_message(format!("You chose {}! Press Enter.", names.join(" & ")), Screen::AdventureHub);
    }

    fn begin_advanced_battle(
        &mut self,
        mut p1: PokemonInstance,
        mut p2: PokemonInstance,
        kind: BattleKind,
        gym_index: Option<u8>,
        weather: Weather,
        trainer_reward: u32,
        elite_index: Option<u8>,
    ) {
        self.save.dex.mark_seen(p1.species_id);
        self.save.dex.mark_seen(p2.species_id);
        if p2.shiny {
            self.save.stats.shinies_found += 1;
        }
        let is_wild = matches!(kind, BattleKind::Wild);
        let can_catch = is_wild;
        let s1 = species_by_id(p1.species_id).map(|s| s.base_stats.speed_at_level(p1.level)).unwrap_or(50) + (p1.iv_speed / 8) as i64;
        let s2 = species_by_id(p2.species_id).map(|s| s.base_stats.speed_at_level(p2.level)).unwrap_or(50) + (p2.iv_speed / 8) as i64;
        let player_first = s1 >= s2;
        let shiny_note = if p2.shiny { " ✦SHINY!" } else { "" };
        let mut log = match kind {
            BattleKind::Wild => vec![
                format!("A wild {}{} (Lv{}) appeared!", p2.display_name(), shiny_note, p2.level),
                format!("Weather: {}", weather.display_name()),
                format!("Go! {}!", p1.display_name()),
            ],
            BattleKind::Gym => vec![format!("Gym Leader sends out {}!", p2.display_name()), format!("Go! {}!", p1.display_name())],
            BattleKind::Exhibition => vec![format!("Exhibition: {} vs {}!", p1.display_name(), p2.display_name())],
            BattleKind::Rival => vec![format!("Rival Blue wants to battle!"), format!("Go! {}!", p1.display_name())],
            BattleKind::Elite => vec![format!("Elite challenges you!"), format!("Go! {}!", p1.display_name())],
            BattleKind::Trainer => vec![format!("Trainer battle!"), format!("Go! {}!", p1.display_name())],
            _ => vec![format!("{} vs {}!", p1.display_name(), p2.display_name())],
        };
        let mut battle = BattleState {
            mode: BattleMode::Advanced, kind,
            player1_max_hp: p1.max_hp, player2_max_hp: p2.max_hp,
            player1: p1.to_legacy(), player2: p2.to_legacy(),
            inst_p1: Some(p1), inst_p2: Some(p2),
            vs_computer: true, is_wild, can_catch, gym_index,
            player1_turn: player_first, move_cursor: 0, battle_menu: 0, in_move_pick: false,
            log, winner_text: None, sprite_lines: Vec::new(), color_sprite_lines: Vec::new(),
            catch_attempts: 0, weather, trainer_reward, elite_index,
        };
        battle.refresh_sprites(self.show_sprites);
        self.battle = Some(battle);
        self.screen = Screen::Battle;
        if !player_first { self.resolve_cpu_turn(); }
    }

    pub fn start_wild_battle(&mut self) {
        if self.save.party.is_empty() { self.screen = Screen::StarterSelect; return; }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("All fainted! Visit the Center.", Screen::AdventureHub); return;
        };
        let route = route_by_id(self.save.current_route).unwrap_or_else(|| default_routes()[0].clone());
        self.save.stats.distance_steps += 1;
        self.save.tick_repel();
        if self.save.repel_steps > 0 {
            self.status = format!("Repel active ({} steps left). No wilds.", self.save.repel_steps);
            return;
        }
        let p1 = self.save.party[lead_idx].clone();
        let wild = wild_on_route(&route);
        self.save.stats.wild_encounters += 1;
        self.begin_advanced_battle(p1, wild, BattleKind::Wild, None, route.weather, 0, None);
    }

    pub fn start_water_battle(&mut self) {
        if self.save.party.is_empty() { return; }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("Heal first!", Screen::AdventureHub); return;
        };
        let route = route_by_id(self.save.current_route).unwrap_or_else(|| default_routes()[0].clone());
        let Some(wild) = water_on_route(&route) else {
            self.status = "No water encounters here. Try Cerulean Coast.".into();
            return;
        };
        let p1 = self.save.party[lead_idx].clone();
        self.save.stats.wild_encounters += 1;
        self.begin_advanced_battle(p1, wild, BattleKind::Wild, None, route.weather, 0, None);
    }

    pub fn start_route_wild(&mut self, route_id: u8) {
        self.save.current_route = route_id;
        self.start_wild_battle();
    }

    pub fn start_route_trainer(&mut self) {
        if self.save.party.is_empty() { return; }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("Heal first!", Screen::AdventureHub); return;
        };
        let trainers = trainers_on_route(self.save.current_route);
        if trainers.is_empty() {
            self.status = "No trainers on this route.".into();
            return;
        }
        let t = &trainers[rand::thread_rng().gen_range(0..trainers.len())];
        let p1 = self.save.party[lead_idx].clone();
        let foe = route_trainer_instance(t);
        let route = route_by_id(self.save.current_route).unwrap_or_else(|| default_routes()[0].clone());
        self.begin_advanced_battle(p1, foe, BattleKind::Trainer, None, route.weather, t.reward_money, None);
    }

    pub fn start_rival_battle(&mut self) {
        if self.save.party.is_empty() { return; }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("Heal first!", Screen::AdventureHub); return;
        };
        let p1 = self.save.party[lead_idx].clone();
        let line = if self.save.starter_line == 0 { 1 } else { self.save.starter_line };
        let foe = rival_instance(self.save.rival_stage, line);
        let reward = 300 + self.save.rival_stage as u32 * 500;
        self.begin_advanced_battle(p1, foe, BattleKind::Rival, None, Weather::Clear, reward, None);
    }

    pub fn start_elite_battle(&mut self, idx: usize) {
        if self.save.party.is_empty() { return; }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("Heal first!", Screen::EliteFour); return;
        };
        if idx as u8 > self.save.elite_progress {
            self.show_message(format!("Beat prior Elite members first (progress {}).", self.save.elite_progress), Screen::EliteFour);
            return;
        }
        let Some(e) = elite_by_index(idx) else { return; };
        let p1 = self.save.party[lead_idx].clone();
        let foe = elite_lead_instance(&e);
        self.begin_advanced_battle(p1, foe, BattleKind::Elite, None, Weather::Clear, e.reward_money, Some(idx as u8));
    }

    pub fn start_gym_battle(&mut self, gym_i: usize) {
        if self.save.party.is_empty() { return; }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("Heal your party first!", Screen::AdventureHub); return;
        };
        let Some(gym) = gym_by_index(gym_i) else { return; };
        if gym_i > 0 && self.save.badge_count() < gym_i as u32 {
            self.show_message(format!("Need {} badges to challenge {}.", gym_i, gym.name), Screen::GymSelect);
            return;
        }
        let p1 = self.save.party[lead_idx].clone();
        let foe = gym_lead_instance(&gym);
        self.begin_advanced_battle(p1, foe, BattleKind::Gym, Some(gym_i as u8), Weather::Clear, gym.reward_money, None);
    }

    pub fn battle_selected_species(&mut self) {
        if self.save.party.is_empty() {
            self.show_message("Start Adventure and pick a starter first.", Screen::StarterSelect);
            return;
        }
        let Some(lead_idx) = self.save.first_conscious() else {
            self.show_message("Heal your party first!", Screen::SpeciesDex); return;
        };
        let Some(sp) = all_species().get(self.species_index) else { return; };
        let p1 = self.save.party[lead_idx].clone();
        let level = (p1.level.saturating_sub(2)).max(5).min(p1.level.saturating_add(5));
        let Some(foe) = exhibition_foe(sp.id, level) else { return; };
        self.begin_advanced_battle(p1, foe, BattleKind::Exhibition, None, Weather::Clear, 0, None);
    }

    pub fn buy_mart_item(&mut self) {
        let catalog = mart_catalog();
        let Some(item) = catalog.get(self.menu_index) else { return; };
        match self.save.buy_item(item.kind, item.price) {
            Ok(m) => { self.status = m; self.persist_save(); }
            Err(e) => self.status = e,
        }
    }

    pub fn tutor_selected_party(&mut self) {
        if self.menu_index >= self.save.party.len() { return; }
        match self.save.party[self.menu_index].tutor_relearn_move() {
            Ok(m) => { self.status = m; self.persist_save(); }
            Err(e) => self.status = e,
        }
    }

    pub fn start_nickname(&mut self) {
        if self.menu_index >= self.save.party.len() { return; }
        self.nickname_target = self.menu_index;
        self.nickname_buf = self.save.party[self.menu_index].nickname.clone();
        self.screen = Screen::NicknameInput;
    }

    pub fn submit_nickname(&mut self) {
        if self.nickname_target < self.save.party.len() {
            self.save.party[self.nickname_target].set_nickname(&self.nickname_buf);
            self.persist_save();
            self.status = format!("Nickname set: {}", self.save.party[self.nickname_target].display_name());
        }
        self.screen = Screen::PartyView;
    }

    pub fn battle_use_potion(&mut self) {
        let Some(b) = &self.battle else { return; };
        if b.winner_text.is_some() || !b.player1_turn { return; }
        let pot_pos = self.save.inventory.iter().position(|i| i.kind.is_heal() && i.count > 0);
        let Some(pot_pos) = pot_pos else {
            if let Some(b) = &mut self.battle { b.log.push("No healing items!".into()); }
            return;
        };
        let kind = self.save.inventory[pot_pos].kind;
        let amt = kind.heal_amount();
        self.save.inventory[pot_pos].count -= 1;
        if self.save.inventory[pot_pos].count == 0 { self.save.inventory.remove(pot_pos); }
        if let Some(b) = &mut self.battle {
            if let Some(p1) = &mut b.inst_p1 {
                let before = p1.current_hp;
                if kind == ItemKind::FullRestore { p1.heal_full(); } else { p1.heal(amt); }
                b.player1.health = p1.current_hp;
                b.log.push(format!("Used {}! {} recovered {} HP.", kind.display_name(), p1.display_name(), p1.current_hp - before));
            }
            b.player1_turn = false;
            b.in_move_pick = false;
        }
        self.resolve_cpu_turn();
    }

    pub fn battle_switch_party(&mut self) {
        let Some(b) = &self.battle else { return; };
        if b.winner_text.is_some() || !b.player1_turn { return; }
        // Find next conscious party member that isn't the current lead species+nick
        let cur_sid = b.inst_p1.as_ref().map(|p| p.species_id).unwrap_or(0);
        let cur_nick = b.inst_p1.as_ref().map(|p| p.nickname.clone()).unwrap_or_default();
        let next = self.save.party.iter().position(|p| !p.is_fainted() && !(p.species_id == cur_sid && p.nickname == cur_nick));
        let Some(ni) = next else {
            if let Some(b) = &mut self.battle { b.log.push("No other conscious Pokemon!".into()); }
            return;
        };
        let new_p = self.save.party[ni].clone();
        if let Some(b) = &mut self.battle {
            // Save old back to party
            if let Some(old) = &b.inst_p1 {
                if let Some(slot) = self.save.party.iter_mut().find(|p| p.species_id == old.species_id && p.nickname == old.nickname) {
                    *slot = old.clone();
                }
            }
            b.log.push(format!("Come back! Go {}!", new_p.display_name()));
            b.player1_max_hp = new_p.max_hp;
            b.player1 = new_p.to_legacy();
            b.inst_p1 = Some(new_p);
            b.player1_turn = false;
            b.in_move_pick = false;
            b.refresh_sprites(self.show_sprites);
        }
        self.resolve_cpu_turn();
    }

    /// Add dex species to party at level 5 (debug/unlock convenience when money >= 500).
    pub fn buy_dex_mon(&mut self) {
        if self.save.party.len() >= 6 {
            self.status = "Party full — store one in the PC box first.".into();
            return;
        }
        if self.save.money < 500 {
            self.status = "Need $500 to recruit from the dex (practice cost).".into();
            return;
        }
        let Some(sp) = all_species().get(self.species_index) else { return; };
        let Some(inst) = PokemonInstance::from_species_id(sp.id, 5) else { return; };
        self.save.money -= 500;
        self.save.dex.mark_caught(sp.id);
        let name = inst.display_name().to_string();
        self.save.party.push(inst);
        self.persist_save();
        self.status = format!("Recruited {name} for $500!");
    }

    pub fn heal_at_center(&mut self) {
        self.save.heal_party_at_center();
        self.persist_save();
        self.show_message("Your Pokemon were fully restored!", Screen::AdventureHub);
    }

    pub fn battle_move_cursor(&mut self, delta: isize) {
        let Some(b) = &self.battle else { return; };
        let len = b.move_labels().len();
        let cur = b.move_cursor;
        if let Some(b) = &mut self.battle { b.move_cursor = move_selection(cur, delta, len); }
    }

    pub fn try_catch(&mut self) {
        let Some(b) = &self.battle else { return; };
        if !b.can_catch || b.winner_text.is_some() { return; }
        // Find a ball in inventory
        let ball_pos = self.save.inventory.iter().position(|i| i.kind.is_ball() && i.count > 0);
        let Some(ball_pos) = ball_pos else {
            if let Some(b) = &mut self.battle { b.log.push("No Poké Balls left!".into()); }
            return;
        };
        let kind = self.save.inventory[ball_pos].kind;
        let ball_mod = kind.catch_modifier();
        self.save.inventory[ball_pos].count -= 1;
        if self.save.inventory[ball_pos].count == 0 { self.save.inventory.remove(ball_pos); }

        let (max_hp, cur_hp, rate, sid, name) = {
            let b = self.battle.as_ref().unwrap();
            let p2 = b.inst_p2.as_ref().unwrap();
            let rate = species_by_id(p2.species_id).map(|s| s.capture_rate).unwrap_or(45);
            (p2.max_hp, p2.current_hp, rate, p2.species_id, p2.display_name().to_string())
        };

        if let Some(b) = &mut self.battle {
            b.log.push(format!("You threw a {}!", kind.display_name()));
            b.catch_attempts += 1;
        }

        if roll_capture(max_hp, cur_hp, rate, ball_mod) {
            if let Some(b) = &mut self.battle {
                b.log.push(format!("Gotcha! {name} was caught!"));
                b.winner_text = Some(format!("Caught {name}!"));
                if let Some(p2) = &b.inst_p2 {
                    let mut caught = p2.clone();
                    caught.heal_full();
                    if self.save.party.len() < 6 {
                        self.save.party.push(caught);
                    } else {
                        self.save.box_storage.push(caught);
                        b.log.push("Party full — sent to PC box.".into());
                    }
                }
            }
            self.save.register_catch(sid);
            self.persist_save();
            self.screen = Screen::BattleOver;
        } else if let Some(b) = &mut self.battle {
            b.log.push("Oh no! It broke free!".into());
            // Foe gets a free turn
            b.player1_turn = false;
            drop(b);
            self.resolve_cpu_turn();
        }
    }

    pub fn try_flee(&mut self) {
        let Some(b) = &self.battle else { return; };
        if !b.is_wild {
            if let Some(b) = &mut self.battle { b.log.push("Can't flee from a trainer!".into()); }
            return;
        }
        let ok = rand::thread_rng().gen_bool(0.6);
        if ok {
            self.save.stats.flee_count += 1;
            if let Some(b) = &mut self.battle {
                b.log.push("Got away safely!".into());
                b.winner_text = Some("Fled.".into());
            }
            self.screen = Screen::BattleOver;
            let _ = write_save(default_save_path(), &self.save);
        } else if let Some(b) = &mut self.battle {
            b.log.push("Can't escape!".into());
            b.player1_turn = false;
            drop(b);
            self.resolve_cpu_turn();
        }
    }

    fn execute_attack(&mut self, move_index: Option<usize>, cpu_prefix: bool) {
        let Some(b) = &mut self.battle else { return; };
        if b.winner_text.is_some() { return; }
        let mode = b.mode;
        let attacker_is_p1 = b.player1_turn;
        let weather = b.weather;

        if mode == BattleMode::Advanced {
            let mi = move_index.unwrap_or(b.move_cursor);
            let attacker_name = b.attacker_name();
            let defender_name = b.defender_name();
            let result = if attacker_is_p1 {
                let (p1, p2) = match (&mut b.inst_p1, &mut b.inst_p2) { (Some(a), Some(d)) => (a, d), _ => return };
                if mi >= p1.moves.len() { return; }
                let mv_name = p1.moves[mi].data.name.clone();
                let mv_desc = if !p1.moves[mi].data.name.is_empty() {
                    format!(" ({} BP)", p1.moves[mi].data.power)
                } else { String::new() };
                b.log.push(format!("{attacker_name} uses {mv_name}!{mv_desc}"));
                execute_move_weather(p1, p2, mi, weather)
            } else {
                let (p2, p1) = match (&mut b.inst_p2, &mut b.inst_p1) { (Some(a), Some(d)) => (a, d), _ => return };
                if mi >= p2.moves.len() { return; }
                let mv_name = p2.moves[mi].data.name.clone();
                b.log.push(format!("Foe {attacker_name} uses {mv_name}!"));
                execute_move_weather(p2, p1, mi, weather)
            };
            if result.critical {
                self.save.stats.critical_hits += 1;
            }
            if result.missed { b.log.push("The attack missed!".into()); }
            else if result.damage_dealt > 0 {
                let crit = if result.critical { " Critical hit!" } else { "" };
                b.log.push(format!("{defender_name} takes {} damage.{crit}", result.damage_dealt));
                if !result.effectiveness_text.is_empty() { b.log.push(result.effectiveness_text); }
            } else if result.effectiveness == 0.0 { b.log.push("It has no effect...".into()); }
            else if !result.flavor.is_empty() { b.log.push(result.flavor); }

            if let Some(p1) = &b.inst_p1 { b.player1.health = p1.current_hp; b.player1_max_hp = p1.max_hp; }
            if let Some(p2) = &b.inst_p2 { b.player2.health = p2.current_hp; b.player2_max_hp = p2.max_hp; }
            if self.check_battle_over() { return; }
            if let Some(b) = &mut self.battle {
                let w = b.weather;
                if let Some(p1) = &mut b.inst_p1 {
                    if let Some(m) = apply_status_residual(p1) { b.log.push(m); b.player1.health = p1.current_hp; }
                    if let Some(m) = apply_weather_residual(p1, w) { b.log.push(m); b.player1.health = p1.current_hp; }
                }
                if let Some(p2) = &mut b.inst_p2 {
                    if let Some(m) = apply_status_residual(p2) { b.log.push(m); b.player2.health = p2.current_hp; }
                    if let Some(m) = apply_weather_residual(p2, w) { b.log.push(m); b.player2.health = p2.current_hp; }
                }
            }
            if self.check_battle_over() { return; }
        } else {
            let moves = if b.player1_turn { b.player1.moves_name.clone() } else { b.player2.moves_name.clone() };
            if moves.is_empty() { return; }
            let mi = move_index.unwrap_or(b.move_cursor).min(moves.len()-1);
            let move_name = &moves[mi];
            let attacker_name = b.attacker_name();
            let defender_name = b.defender_name();
            b.log.push(format!("{attacker_name} uses {move_name}!"));
            if roll_block() { b.log.push(format!("{defender_name} blocked!")); }
            else {
                let crit = roll_critical_chance();
                let result = if attacker_is_p1 { b.player2.take_hit(crit) } else { b.player1.take_hit(crit) };
                let c = if result.critical { " Critical!" } else { "" };
                b.log.push(format!("{defender_name} takes {} dmg.{c}", result.damage_dealt));
            }
            if self.check_battle_over() { return; }
        }

        if let Some(b) = &mut self.battle {
            b.player1_turn = !b.player1_turn;
            b.move_cursor = 0;
            b.in_move_pick = b.player1_turn;
            b.refresh_sprites(self.show_sprites);
        }
        if let Some(b) = &self.battle {
            if !cpu_prefix && b.vs_computer && !b.player1_turn && b.winner_text.is_none() {
                self.resolve_cpu_turn();
            }
        }
    }

    pub fn player_attack(&mut self) { self.execute_attack(None, false); }

    fn resolve_cpu_turn(&mut self) {
        let Some(b) = &self.battle else { return; };
        if b.winner_text.is_some() || b.player1_turn { return; }
        let moves_len = match b.mode {
            BattleMode::Classic => b.player2.moves_name.len(),
            BattleMode::Advanced => b.inst_p2.as_ref().map(|p| p.moves.len()).unwrap_or(0),
        };
        if moves_len == 0 { self.execute_attack(Some(0), true); return; }
        // AI: prefer super-effective damaging moves; status if player has no status
        let mi = if b.mode == BattleMode::Advanced {
            let b = self.battle.as_ref().unwrap();
            let p2 = b.inst_p2.as_ref().unwrap();
            let p1 = b.inst_p1.as_ref().unwrap();
            let mut best = 0usize;
            let mut best_score = -1.0f64;
            for (i, m) in p2.moves.iter().enumerate() {
                if !m.can_use() { continue; }
                let mult = pokemon_text_game::pokemon::type_effectiveness(m.data.move_type, &p1.types);
                let mut score = mult * m.data.power as f64;
                // Prefer status if player healthy and move is status
                if m.data.power == 0 && p1.status == pokemon_text_game::pokemon::StatusCondition::None {
                    score = 30.0;
                }
                if score > best_score { best_score = score; best = i; }
            }
            best
        } else {
            rand::thread_rng().gen_range(0..moves_len)
        };
        self.execute_attack(Some(mi), true);
    }

    fn check_battle_over(&mut self) -> bool {
        let Some(b) = &self.battle else { return false; };
        let (p1_faint, p2_faint) = match b.mode {
            BattleMode::Classic => (b.player1.is_fainted(), b.player2.is_fainted()),
            BattleMode::Advanced => (
                b.inst_p1.as_ref().map(|p| p.is_fainted()).unwrap_or(true),
                b.inst_p2.as_ref().map(|p| p.is_fainted()).unwrap_or(true),
            ),
        };
        let kind = b.kind;
        let gym_i = b.gym_index;
        let elite_i = b.elite_index;
        let trainer_reward = b.trainer_reward;
        let leg = b.inst_p2.as_ref().map(|p| p.species_id >= 144 && p.species_id <= 151).unwrap_or(false);
        let mode_adv = b.mode == BattleMode::Advanced;
        let is_wild = b.is_wild;
        let p2_name = b.player2.name.clone();
        let p2_species = b.inst_p2.as_ref().map(|p| p.species_id);
        let p1_inst = b.inst_p1.clone();
        let p2_inst = b.inst_p2.clone();

        let winner = if p1_faint {
            Some(if is_wild { format!("Defeated by wild {p2_name}!") } else { format!("You lost to {p2_name}!") })
        } else if p2_faint {
            Some(match kind {
                BattleKind::Wild => format!("Defeated wild {p2_name}!"),
                BattleKind::Gym => "Gym victory!".into(),
                BattleKind::Exhibition => format!("Exhibition win vs {p2_name}!"),
                BattleKind::Rival => "Rival defeated!".into(),
                BattleKind::Elite => "Elite member defeated!".into(),
                BattleKind::Trainer => "Trainer defeated!".into(),
                _ => "You win!".into(),
            })
        } else {
            None
        };

        let Some(winner) = winner else { return false; };

        let mut extra_logs = vec![winner.clone()];
        if mode_adv {
            if let Some(ref p1) = p1_inst {
                if let Some(slot) = self.save.party.iter_mut().find(|p| p.species_id == p1.species_id && p.nickname == p1.nickname) {
                    *slot = p1.clone();
                }
            }
            if p2_faint && !p1_faint {
                self.save.register_win(leg);
                // XP to all conscious party (spread)
                if let Some(p2) = &p2_inst {
                    let base = species_by_id(p2.species_id).map(|s| s.base_experience).unwrap_or(64);
                    let xp = p2.xp_for_defeating(base);
                    let party_len = self.save.party.len().max(1) as u32;
                    let share = xp / party_len + xp / 4;
                    let mut evo_count = 0u32;
                    for p in &mut self.save.party {
                        if !p.is_fainted() {
                            let msgs = p.gain_xp(share);
                            for m in &msgs {
                                if m.contains("evolving") || m.contains("evolved") {
                                    evo_count += 1;
                                }
                            }
                            extra_logs.extend(msgs);
                        }
                    }
                    self.save.stats.evolutions += evo_count;
                }
                let mut reward = trainer_reward.max(50 + rand::thread_rng().gen_range(0..80));
                if let Some(gi) = gym_i {
                    if let Some(g) = gym_by_index(gi as usize) {
                        reward = g.reward_money;
                        self.save.register_gym_win(gi);
                        extra_logs.push(format!("Earned the {}!", g.badge_name));
                    }
                }
                if matches!(kind, BattleKind::Trainer | BattleKind::Rival) {
                    self.save.stats.trainers_defeated += 1;
                }
                if matches!(kind, BattleKind::Rival) {
                    self.save.rival_stage = (self.save.rival_stage + 1).min(2);
                    extra_logs.push("Rival: \"I'll get you next time!\"".into());
                }
                if let Some(ei) = elite_i {
                    self.save.stats.elite_wins += 1;
                    if ei + 1 > self.save.elite_progress {
                        self.save.elite_progress = ei + 1;
                    }
                    if ei >= 4 {
                        extra_logs.push("You are the Champion!".into());
                    }
                }
                let day = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 86400) as u32;
                let daily = daily_challenge_for_day(day);
                if p2_species == Some(daily.target_species_id) {
                    reward += daily.bonus_money;
                    extra_logs.push(format!("Daily challenge bonus +${}!", daily.bonus_money));
                }
                self.save.money += reward;
                self.save.stats.money_earned += reward;
                extra_logs.push(format!("Got ${reward}!"));
            } else if p1_faint {
                self.save.register_loss();
            }
            let _ = write_save(default_save_path(), &self.save);
        }

        if let Some(b) = &mut self.battle {
            b.winner_text = Some(winner);
            b.log.extend(extra_logs);
        }
        self.screen = Screen::BattleOver;
        true
    }

    pub fn end_battle_to_menu(&mut self) {
        let ret = if self.battle.as_ref().map(|b| b.mode == BattleMode::Advanced).unwrap_or(false) {
            Screen::AdventureHub
        } else { Screen::MainMenu };
        self.battle = None; self.menu_index = 0; self.screen = ret;
    }

    pub fn toggle_sprites(&mut self) {
        self.show_sprites = !self.show_sprites;
        self.save.settings.show_sprites = self.show_sprites;
        if let Some(b) = &mut self.battle { b.refresh_sprites(self.show_sprites); }
        self.status = format!("Sprites: {}", if self.show_sprites { "ON" } else { "OFF" });
    }

    pub fn cycle_dex_filter(&mut self) {
        self.dex_filter = match self.dex_filter {
            DexFilter::All => DexFilter::Seen,
            DexFilter::Seen => DexFilter::Caught,
            DexFilter::Caught => DexFilter::All,
            DexFilter::ByType(_) => DexFilter::All,
        };
        let idxs = self.filtered_species_indices();
        if let Some(&i) = idxs.first() { self.species_index = i; }
        self.status = format!("Dex filter: {:?}", self.dex_filter);
    }

    pub fn adjust_volume(&mut self, delta: f32) {
        let v = (self.save.settings.music_volume + delta).clamp(0.0, 1.0);
        self.save.settings.music_volume = v;
        self.pending_audio_volume = Some(v);
        self.status = format!("Volume: {:.0}%", v * 100.0);
    }

    /// Jump dex to species id (cheat/debug: type number then g in search, or Home+id).
    pub fn jump_dex_to_id(&mut self, id: u16) {
        if let Some(i) = all_species().iter().position(|s| s.id == id) {
            self.species_index = i;
            self.save.dex.mark_seen(id);
            self.status = format!("Jumped to #{id}");
        }
    }
}
