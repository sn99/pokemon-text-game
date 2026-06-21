/*MIT License

Copyright (c) 2018 sn99

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rand::Rng;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};

use pokemon_text_game::extra::{clamp_index, move_selection, parse_i64, parse_moves};
use pokemon_text_game::{
    read_team_from_file, roll_block, roll_critical_chance, write_team_to_file, Pokemon,
    PokemonsList, TEAM_PATH,
};

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    MainMenu,
    PlayMode,
    SelectPokemon { slot: SelectSlot },
    Battle,
    BattleOver,
    PokedexMenu,
    PokedexList { action: PokedexAction },
    FormInput,
    Message,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectSlot {
    VsComputerPlayer,
    VsHumanP1,
    VsHumanP2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PokedexAction {
    Edit,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormKind {
    CreatePokemon,
    EditPokemon { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormField {
    Name,
    Moves,
    Health,
    Type,
}

impl FormField {
    fn next(self) -> Self {
        match self {
            Self::Name => Self::Moves,
            Self::Moves => Self::Health,
            Self::Health => Self::Type,
            Self::Type => Self::Name,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Name => Self::Type,
            Self::Moves => Self::Name,
            Self::Health => Self::Moves,
            Self::Type => Self::Health,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Moves => "Moves (space-separated)",
            Self::Health => "Health",
            Self::Type => "Type (number)",
        }
    }
}

struct BattleState {
    player1: Pokemon,
    player2: Pokemon,
    player1_max_hp: i64,
    player2_max_hp: i64,
    vs_computer: bool,
    player1_turn: bool,
    move_cursor: usize,
    log: Vec<String>,
    winner_text: Option<String>,
}

struct FormState {
    kind: FormKind,
    field: FormField,
    name: String,
    moves: String,
    health: String,
    pokemon_type: String,
    error: Option<String>,
}

struct App {
    screen: Screen,
    team: PokemonsList,
    list_state: ListState,
    menu_index: usize,
    status: String,
    message: String,
    message_return: Screen,
    form: Option<FormState>,
    battle: Option<BattleState>,
    pending_select: Option<SelectSlot>,
    selected_p1_index: Option<usize>,
    should_quit: bool,
    load_error: Option<String>,
}

impl App {
    fn new() -> Self {
        let (team, load_error) = match read_team_from_file(TEAM_PATH) {
            Ok(t) => (t, None),
            Err(e) => (
                PokemonsList::default(),
                Some(format!("Could not load {TEAM_PATH}: {e}")),
            ),
        };

        let mut list_state = ListState::default();
        if !team.pokeball.is_empty() {
            list_state.select(Some(0));
        }

        let mut app = Self {
            screen: Screen::MainMenu,
            team,
            list_state,
            menu_index: 0,
            status: String::new(),
            message: String::new(),
            message_return: Screen::MainMenu,
            form: None,
            battle: None,
            pending_select: None,
            selected_p1_index: None,
            should_quit: false,
            load_error,
        };

        if let Some(err) = app.load_error.clone() {
            app.show_message(err, Screen::MainMenu);
        }

        app
    }

    fn save_team(&mut self) -> bool {
        match write_team_to_file(TEAM_PATH, &self.team) {
            Ok(()) => {
                self.status = "Saved.".into();
                true
            }
            Err(e) => {
                self.status = format!("Save failed: {e}");
                false
            }
        }
    }

    fn show_message(&mut self, msg: impl Into<String>, ret: Screen) {
        self.message = msg.into();
        self.message_return = ret;
        self.screen = Screen::Message;
    }

    fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn ensure_list_selection(&mut self) {
        let len = self.team.pokeball.len();
        if len == 0 {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(i) = self.list_state.selected() {
            self.list_state.select(Some(clamp_index(i, len)));
        }
    }

    fn move_list(&mut self, delta: isize) {
        let len = self.team.pokeball.len();
        if len == 0 {
            return;
        }
        let cur = self.list_state.selected().unwrap_or(0);
        let next = move_selection(cur, delta, len);
        self.list_state.select(Some(next));
    }

    fn start_form_create(&mut self) {
        self.form = Some(FormState {
            kind: FormKind::CreatePokemon,
            field: FormField::Name,
            name: String::new(),
            moves: String::new(),
            health: "400".into(),
            pokemon_type: "1".into(),
            error: None,
        });
        self.screen = Screen::FormInput;
    }

    fn start_form_edit(&mut self, index: usize) {
        let p = &self.team.pokeball[index];
        self.form = Some(FormState {
            kind: FormKind::EditPokemon { index },
            field: FormField::Name,
            name: p.name.clone(),
            moves: p.moves_name.join(" "),
            health: p.health.to_string(),
            pokemon_type: p.pokemon_type.to_string(),
            error: None,
        });
        self.screen = Screen::FormInput;
    }

    fn submit_form(&mut self) {
        let Some(form) = self.form.as_ref() else {
            return;
        };

        let name = form.name.trim().to_string();
        if name.is_empty() {
            if let Some(f) = self.form.as_mut() {
                f.error = Some("Name cannot be empty.".into());
            }
            return;
        }

        let moves = parse_moves(&form.moves);
        if moves.is_empty() {
            if let Some(f) = self.form.as_mut() {
                f.error = Some("Enter at least one move (space-separated).".into());
            }
            return;
        }

        let health = match parse_i64(&form.health) {
            Ok(h) if h > 0 => h,
            Ok(_) => {
                if let Some(f) = self.form.as_mut() {
                    f.error = Some("Health must be a positive number.".into());
                }
                return;
            }
            Err(e) => {
                if let Some(f) = self.form.as_mut() {
                    f.error = Some(e);
                }
                return;
            }
        };

        let pokemon_type = match parse_i64(&form.pokemon_type) {
            Ok(t) => t,
            Err(e) => {
                if let Some(f) = self.form.as_mut() {
                    f.error = Some(e);
                }
                return;
            }
        };

        let kind = form.kind;
        match kind {
            FormKind::CreatePokemon => {
                self.team
                    .pokeball
                    .push(Pokemon::with_stats(name, moves, health, pokemon_type));
                self.save_team();
                self.form = None;
                self.ensure_list_selection();
                let idx = self.team.pokeball.len().saturating_sub(1);
                self.list_state.select(Some(idx));
                self.show_message("Pokemon created!", Screen::PokedexMenu);
            }
            FormKind::EditPokemon { index } => {
                if let Some(p) = self.team.pokeball.get_mut(index) {
                    p.edit_full(name, moves, health, pokemon_type);
                }
                self.save_team();
                self.form = None;
                self.show_message("Pokemon updated!", Screen::PokedexMenu);
            }
        }
    }

    fn delete_selected(&mut self) {
        let Some(idx) = self.selected_index() else {
            self.status = "No Pokemon selected.".into();
            return;
        };
        if idx >= self.team.pokeball.len() {
            return;
        }
        let name = self.team.pokeball[idx].name.clone();
        self.team.pokeball.remove(idx);
        self.save_team();
        self.ensure_list_selection();
        self.show_message(format!("Deleted {name}."), Screen::PokedexMenu);
    }

    fn begin_pokemon_select(&mut self, slot: SelectSlot) {
        if self.team.pokeball.is_empty() {
            self.show_message(
                "No Pokemon in the Pokedex. Create one first.",
                Screen::MainMenu,
            );
            return;
        }
        self.pending_select = Some(slot);
        self.ensure_list_selection();
        self.screen = Screen::SelectPokemon { slot };
    }

    fn confirm_pokemon_select(&mut self) {
        let Some(slot) = self.pending_select else {
            return;
        };
        let Some(idx) = self.selected_index() else {
            return;
        };
        if idx >= self.team.pokeball.len() {
            return;
        }

        match slot {
            SelectSlot::VsComputerPlayer => {
                let player = self.team.pokeball[idx].clone();
                let mut rng = rand::thread_rng();
                let cpu_idx = rng.gen_range(0..self.team.pokeball.len());
                let cpu = self.team.pokeball[cpu_idx].clone();
                let player_first = rng.gen_range(0..2) == 0;
                self.start_battle(player, cpu, true, player_first);
            }
            SelectSlot::VsHumanP1 => {
                self.selected_p1_index = Some(idx);
                self.begin_pokemon_select(SelectSlot::VsHumanP2);
            }
            SelectSlot::VsHumanP2 => {
                let p1_idx = self.selected_p1_index.unwrap_or(0);
                let p1 = self.team.pokeball[p1_idx].clone();
                let p2 = self.team.pokeball[idx].clone();
                let player_first = rand::thread_rng().gen_range(0..2) == 0;
                self.selected_p1_index = None;
                self.start_battle(p1, p2, false, player_first);
            }
        }
    }

    fn start_battle(&mut self, p1: Pokemon, p2: Pokemon, vs_computer: bool, player1_first: bool) {
        let p1_max = p1.health;
        let p2_max = p2.health;
        let mut log = vec![
            format!("{} vs {}!", p1.name, p2.name),
            if player1_first {
                format!("{} goes first!", p1.name)
            } else {
                format!("{} goes first!", p2.name)
            },
        ];
        if vs_computer {
            log.push(format!("Computer chose {}.", p2.name));
        }

        self.battle = Some(BattleState {
            player1: p1,
            player2: p2,
            player1_max_hp: p1_max,
            player2_max_hp: p2_max,
            vs_computer,
            player1_turn: player1_first,
            move_cursor: 0,
            log,
            winner_text: None,
        });
        self.pending_select = None;
        self.screen = Screen::Battle;

        // If CPU goes first, resolve its opening turn immediately.
        if !player1_first && vs_computer {
            self.resolve_cpu_turn();
        }
    }

    fn current_attacker_moves(&self) -> Vec<String> {
        let Some(b) = &self.battle else {
            return Vec::new();
        };
        if b.player1_turn {
            b.player1.moves_name.clone()
        } else {
            b.player2.moves_name.clone()
        }
    }

    fn battle_move_up(&mut self) {
        let moves = self.current_attacker_moves();
        if let Some(b) = &mut self.battle {
            b.move_cursor = move_selection(b.move_cursor, -1, moves.len());
        }
    }

    fn battle_move_down(&mut self) {
        let moves = self.current_attacker_moves();
        if let Some(b) = &mut self.battle {
            b.move_cursor = move_selection(b.move_cursor, 1, moves.len());
        }
    }

    fn player_attack(&mut self) {
        let Some(b) = &mut self.battle else {
            return;
        };
        if b.winner_text.is_some() {
            return;
        }

        let attacker_is_p1 = b.player1_turn;
        let moves = if attacker_is_p1 {
            b.player1.moves_name.clone()
        } else {
            b.player2.moves_name.clone()
        };
        if moves.is_empty() {
            b.log.push("No moves available!".into());
            return;
        }
        let mi = b.move_cursor.min(moves.len() - 1);
        let move_name = moves[mi].clone();
        let attacker_name = if attacker_is_p1 {
            b.player1.name.clone()
        } else {
            b.player2.name.clone()
        };
        let defender_name = if attacker_is_p1 {
            b.player2.name.clone()
        } else {
            b.player1.name.clone()
        };

        b.log.push(format!("{attacker_name} uses {move_name}!"));

        if roll_block() {
            b.log.push(format!("{defender_name} blocked it!"));
        } else {
            let crit = roll_critical_chance();
            let result = if attacker_is_p1 {
                b.player2.take_hit(crit)
            } else {
                b.player1.take_hit(crit)
            };
            let crit_txt = if result.critical { " Critical!" } else { "" };
            b.log.push(format!(
                "{} takes {} damage.{} ({})",
                defender_name, result.damage_dealt, crit_txt, result.flavor
            ));
        }

        if self.check_battle_over() {
            return;
        }

        // Switch turn
        if let Some(b) = &mut self.battle {
            b.player1_turn = !b.player1_turn;
            b.move_cursor = 0;
        }

        // CPU auto-turn when applicable
        if let Some(b) = &self.battle {
            if b.vs_computer && !b.player1_turn && b.winner_text.is_none() {
                self.resolve_cpu_turn();
            }
        }
    }

    fn resolve_cpu_turn(&mut self) {
        let Some(b) = &mut self.battle else {
            return;
        };
        if b.winner_text.is_some() || b.player1_turn {
            return;
        }

        let moves = b.player2.moves_name.clone();
        if moves.is_empty() {
            b.log.push("CPU has no moves!".into());
            b.player1_turn = true;
            return;
        }
        let mi = rand::thread_rng().gen_range(0..moves.len());
        let move_name = moves[mi].clone();
        let attacker_name = b.player2.name.clone();
        let defender_name = b.player1.name.clone();

        b.log.push(format!("CPU's {attacker_name} uses {move_name}!"));

        if roll_block() {
            b.log.push(format!("{defender_name} blocked it!"));
        } else {
            let crit = roll_critical_chance();
            let result = b.player1.take_hit(crit);
            let crit_txt = if result.critical { " Critical!" } else { "" };
            b.log.push(format!(
                "{} takes {} damage.{} ({})",
                defender_name, result.damage_dealt, crit_txt, result.flavor
            ));
        }

        if self.check_battle_over() {
            return;
        }

        if let Some(b) = &mut self.battle {
            b.player1_turn = true;
            b.move_cursor = 0;
        }
    }

    fn check_battle_over(&mut self) -> bool {
        let Some(b) = &mut self.battle else {
            return false;
        };
        if b.player1.is_fainted() {
            let winner = if b.vs_computer {
                format!("Computer wins with {}!", b.player2.name)
            } else {
                format!("Player 2 wins with {}!", b.player2.name)
            };
            b.winner_text = Some(winner.clone());
            b.log.push(winner);
            self.screen = Screen::BattleOver;
            return true;
        }
        if b.player2.is_fainted() {
            let winner = if b.vs_computer {
                format!("You win with {}!", b.player1.name)
            } else {
                format!("Player 1 wins with {}!", b.player1.name)
            };
            b.winner_text = Some(winner.clone());
            b.log.push(winner);
            self.screen = Screen::BattleOver;
            return true;
        }
        false
    }

    fn end_battle_to_menu(&mut self) {
        self.battle = None;
        self.screen = Screen::MainMenu;
        self.menu_index = 0;
    }
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

impl App {
    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Global quit with Ctrl+C
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        match self.screen {
            Screen::MainMenu => self.key_main_menu(key),
            Screen::PlayMode => self.key_play_mode(key),
            Screen::SelectPokemon { .. } => self.key_select_pokemon(key),
            Screen::Battle => self.key_battle(key),
            Screen::BattleOver => self.key_battle_over(key),
            Screen::PokedexMenu => self.key_pokedex_menu(key),
            Screen::PokedexList { action } => self.key_pokedex_list(key, action),
            Screen::FormInput => self.key_form(key),
            Screen::Message => self.key_message(key),
        }
    }

    fn key_main_menu(&mut self, key: KeyEvent) {
        const ITEMS: usize = 3;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_index = move_selection(self.menu_index, -1, ITEMS);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = move_selection(self.menu_index, 1, ITEMS);
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.menu_index {
                0 => {
                    self.menu_index = 0;
                    self.screen = Screen::PlayMode;
                }
                1 => {
                    self.menu_index = 0;
                    self.screen = Screen::PokedexMenu;
                }
                _ => self.should_quit = true,
            },
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') => {
                self.menu_index = 0;
                self.screen = Screen::PlayMode;
            }
            KeyCode::Char('2') => {
                self.menu_index = 0;
                self.screen = Screen::PokedexMenu;
            }
            KeyCode::Char('3') => self.should_quit = true,
            _ => {}
        }
    }

    fn key_play_mode(&mut self, key: KeyEvent) {
        const ITEMS: usize = 3;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_index = move_selection(self.menu_index, -1, ITEMS);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = move_selection(self.menu_index, 1, ITEMS);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.menu_index = 0;
                self.screen = Screen::MainMenu;
            }
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('1') | KeyCode::Char('2') => {
                let choice = if matches!(key.code, KeyCode::Char('1')) {
                    0
                } else if matches!(key.code, KeyCode::Char('2')) {
                    1
                } else {
                    self.menu_index
                };
                match choice {
                    0 => self.begin_pokemon_select(SelectSlot::VsComputerPlayer),
                    1 => self.begin_pokemon_select(SelectSlot::VsHumanP1),
                    _ => {
                        self.menu_index = 0;
                        self.screen = Screen::MainMenu;
                    }
                }
            }
            KeyCode::Char('3') => {
                self.menu_index = 0;
                self.screen = Screen::MainMenu;
            }
            _ => {}
        }
    }

    fn key_select_pokemon(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_list(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_list(1),
            KeyCode::Enter | KeyCode::Char(' ') => self.confirm_pokemon_select(),
            KeyCode::Esc | KeyCode::Char('q') => {
                self.pending_select = None;
                self.selected_p1_index = None;
                self.menu_index = 0;
                self.screen = Screen::PlayMode;
            }
            _ => {}
        }
    }

    fn key_battle(&mut self, key: KeyEvent) {
        let Some(b) = &self.battle else {
            return;
        };
        // During PvP, player2 controls when it's their turn; same keys.
        // During vs CPU, only player1 acts interactively.
        if b.vs_computer && !b.player1_turn {
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.battle_move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.battle_move_down(),
            KeyCode::Enter | KeyCode::Char(' ') => self.player_attack(),
            KeyCode::Esc | KeyCode::Char('q') => {
                self.battle = None;
                self.screen = Screen::MainMenu;
                self.menu_index = 0;
            }
            _ => {}
        }
    }

    fn key_battle_over(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q') => {
                self.end_battle_to_menu();
            }
            _ => {}
        }
    }

    fn key_pokedex_menu(&mut self, key: KeyEvent) {
        const ITEMS: usize = 4;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_index = move_selection(self.menu_index, -1, ITEMS);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_index = move_selection(self.menu_index, 1, ITEMS);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.menu_index = 0;
                self.screen = Screen::MainMenu;
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.menu_index {
                0 => self.start_form_create(),
                1 => {
                    if self.team.pokeball.is_empty() {
                        self.status = "No Pokemon to edit.".into();
                    } else {
                        self.ensure_list_selection();
                        self.screen = Screen::PokedexList {
                            action: PokedexAction::Edit,
                        };
                    }
                }
                2 => {
                    if self.team.pokeball.is_empty() {
                        self.status = "No Pokemon to delete.".into();
                    } else {
                        self.ensure_list_selection();
                        self.screen = Screen::PokedexList {
                            action: PokedexAction::Delete,
                        };
                    }
                }
                _ => {
                    self.menu_index = 0;
                    self.screen = Screen::MainMenu;
                }
            },
            KeyCode::Char('1') => self.start_form_create(),
            KeyCode::Char('2') => {
                if !self.team.pokeball.is_empty() {
                    self.ensure_list_selection();
                    self.screen = Screen::PokedexList {
                        action: PokedexAction::Edit,
                    };
                }
            }
            KeyCode::Char('3') => {
                if !self.team.pokeball.is_empty() {
                    self.ensure_list_selection();
                    self.screen = Screen::PokedexList {
                        action: PokedexAction::Delete,
                    };
                }
            }
            KeyCode::Char('4') => {
                self.menu_index = 0;
                self.screen = Screen::MainMenu;
            }
            _ => {}
        }
    }

    fn key_pokedex_list(&mut self, key: KeyEvent, action: PokedexAction) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_list(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_list(1),
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::PokedexMenu;
            }
            KeyCode::Enter | KeyCode::Char(' ') => match action {
                PokedexAction::Edit => {
                    if let Some(i) = self.selected_index() {
                        self.start_form_edit(i);
                    }
                }
                PokedexAction::Delete => self.delete_selected(),
            },
            _ => {}
        }
    }

    fn key_form(&mut self, key: KeyEvent) {
        let Some(form) = self.form.as_mut() else {
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.form = None;
                self.screen = Screen::PokedexMenu;
            }
            KeyCode::Tab | KeyCode::Down => {
                form.field = form.field.next();
                form.error = None;
            }
            KeyCode::BackTab | KeyCode::Up => {
                form.field = form.field.prev();
                form.error = None;
            }
            KeyCode::Enter => {
                self.submit_form();
            }
            KeyCode::Backspace => {
                form.error = None;
                let buf = match form.field {
                    FormField::Name => &mut form.name,
                    FormField::Moves => &mut form.moves,
                    FormField::Health => &mut form.health,
                    FormField::Type => &mut form.pokemon_type,
                };
                buf.pop();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                form.error = None;
                let buf = match form.field {
                    FormField::Name => &mut form.name,
                    FormField::Moves => &mut form.moves,
                    FormField::Health => &mut form.health,
                    FormField::Type => &mut form.pokemon_type,
                };
                buf.push(c);
            }
            _ => {}
        }
    }

    fn key_message(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = self.message_return;
                self.message.clear();
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn menu_items<'a>(labels: &[&'a str], selected: usize) -> Vec<ListItem<'a>> {
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let prefix = if i == selected { "> " } else { "  " };
            let style = if i == selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(
                format!("{prefix}{}. {}", i + 1, label),
                style,
            )))
        })
        .collect()
}

fn help_line(text: &str) -> Paragraph<'_> {
    Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
}

impl App {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(2),
            ])
            .split(area);

        let title = Paragraph::new("Pokemon Text Game")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        match self.screen {
            Screen::MainMenu => self.draw_main_menu(frame, chunks[1]),
            Screen::PlayMode => self.draw_play_mode(frame, chunks[1]),
            Screen::SelectPokemon { slot } => self.draw_select_pokemon(frame, chunks[1], slot),
            Screen::Battle | Screen::BattleOver => self.draw_battle(frame, chunks[1]),
            Screen::PokedexMenu => self.draw_pokedex_menu(frame, chunks[1]),
            Screen::PokedexList { action } => self.draw_pokedex_list(frame, chunks[1], action),
            Screen::FormInput => self.draw_form(frame, chunks[1]),
            Screen::Message => self.draw_message_popup(frame, area),
        }

        let status_text = if !self.status.is_empty() {
            self.status.as_str()
        } else {
            "↑/↓ navigate · Enter select · q/Esc back · Ctrl+C quit"
        };
        frame.render_widget(help_line(status_text), chunks[2]);
    }

    fn draw_main_menu(&self, frame: &mut Frame, area: Rect) {
        let labels = ["Play game", "Enter Pokedex", "Exit"];
        let items = menu_items(&labels, self.menu_index);
        let list = List::new(items).block(
            Block::default()
                .title(" Main Menu ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(list, area);
    }

    fn draw_play_mode(&self, frame: &mut Frame, area: Rect) {
        let labels = ["Against Computer", "Against Human", "Back"];
        let items = menu_items(&labels, self.menu_index);
        let list = List::new(items).block(
            Block::default()
                .title(" Play Mode ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(list, area);
    }

    fn draw_select_pokemon(&mut self, frame: &mut Frame, area: Rect, slot: SelectSlot) {
        let title = match slot {
            SelectSlot::VsComputerPlayer => " Choose your Pokemon ",
            SelectSlot::VsHumanP1 => " Player 1 — choose Pokemon ",
            SelectSlot::VsHumanP2 => " Player 2 — choose Pokemon ",
        };
        let items: Vec<ListItem> = self
            .team
            .pokeball
            .iter()
            .enumerate()
            .map(|(i, p)| {
                ListItem::new(format!(
                    "{}. {}  HP:{}  moves:{}",
                    i + 1,
                    p.name,
                    p.health,
                    p.moves_name.join(", ")
                ))
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_battle(&self, frame: &mut Frame, area: Rect) {
        let Some(b) = &self.battle else {
            return;
        };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(4),
                Constraint::Length(8),
            ])
            .split(area);

        let p1_ratio = b.player1.health_ratio(b.player1_max_hp);
        let p2_ratio = b.player2.health_ratio(b.player2_max_hp);

        let p1_label = if b.vs_computer { "You" } else { "Player 1" };
        let p2_label = if b.vs_computer {
            "Computer"
        } else {
            "Player 2"
        };

        let p1_gauge = Gauge::default()
            .block(Block::default().title(format!(
                " {p1_label}: {} ",
                b.player1.name
            )))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(p1_ratio)
            .label(format!(
                "{} / {}",
                b.player1.health.max(0),
                b.player1_max_hp
            ));
        frame.render_widget(p1_gauge, rows[0]);

        let p2_gauge = Gauge::default()
            .block(Block::default().title(format!(
                " {p2_label}: {} ",
                b.player2.name
            )))
            .gauge_style(Style::default().fg(Color::Red))
            .ratio(p2_ratio)
            .label(format!(
                "{} / {}",
                b.player2.health.max(0),
                b.player2_max_hp
            ));
        frame.render_widget(p2_gauge, rows[1]);

        // Battle log
        let log_lines: Vec<Line> = b
            .log
            .iter()
            .rev()
            .take(12)
            .rev()
            .map(|s| Line::from(s.as_str()))
            .collect();
        let log = Paragraph::new(log_lines)
            .block(Block::default().title(" Battle Log ").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(log, rows[2]);

        if self.screen == Screen::BattleOver {
            let win = b
                .winner_text
                .clone()
                .unwrap_or_else(|| "Battle over".into());
            let msg = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    win,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Press Enter to return to main menu"),
            ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Result ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
            frame.render_widget(msg, rows[3]);
        } else {
            let whose = if b.player1_turn {
                if b.vs_computer {
                    "Your turn — choose a move"
                } else {
                    "Player 1's turn — choose a move"
                }
            } else if b.vs_computer {
                "Computer is attacking..."
            } else {
                "Player 2's turn — choose a move"
            };

            let moves = if b.player1_turn {
                &b.player1.moves_name
            } else {
                &b.player2.moves_name
            };
            let items: Vec<ListItem> = moves
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let prefix = if i == b.move_cursor { "> " } else { "  " };
                    let style = if i == b.move_cursor {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(
                        format!("{prefix}{}. {m}", i + 1),
                        style,
                    )))
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .title(format!(" Moves — {whose} "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
            frame.render_widget(list, rows[3]);
        }
    }

    fn draw_pokedex_menu(&self, frame: &mut Frame, area: Rect) {
        let labels = [
            "Create new Pokemon",
            "Edit existing Pokemon",
            "Delete a Pokemon",
            "Main menu",
        ];
        let items = menu_items(&labels, self.menu_index);
        let list = List::new(items).block(
            Block::default()
                .title(" Pokedex ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue)),
        );
        frame.render_widget(list, area);
    }

    fn draw_pokedex_list(&mut self, frame: &mut Frame, area: Rect, action: PokedexAction) {
        let title = match action {
            PokedexAction::Edit => " Select Pokemon to edit ",
            PokedexAction::Delete => " Select Pokemon to delete ",
        };
        let items: Vec<ListItem> = self
            .team
            .pokeball
            .iter()
            .enumerate()
            .map(|(i, p)| {
                ListItem::new(format!(
                    "{}. {}  HP:{}  type:{}  moves:{}",
                    i + 1,
                    p.name,
                    p.health,
                    p.pokemon_type,
                    p.moves_name.join(", ")
                ))
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_form(&self, frame: &mut Frame, area: Rect) {
        let Some(form) = &self.form else {
            return;
        };
        let title = match form.kind {
            FormKind::CreatePokemon => " Create Pokemon ",
            FormKind::EditPokemon { .. } => " Edit Pokemon ",
        };

        let field_style = |f: FormField| {
            if form.field == f {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            }
        };
        let val = |f: FormField, s: &str| {
            let display = if s.is_empty() { " " } else { s };
            let cursor = if form.field == f { "▌" } else { "" };
            format!("{display}{cursor}")
        };

        let lines = vec![
            Line::from(Span::styled(
                format!("  {}:", FormField::Name.label()),
                field_style(FormField::Name),
            )),
            Line::from(format!("    {}", val(FormField::Name, &form.name))),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}:", FormField::Moves.label()),
                field_style(FormField::Moves),
            )),
            Line::from(format!("    {}", val(FormField::Moves, &form.moves))),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}:", FormField::Health.label()),
                field_style(FormField::Health),
            )),
            Line::from(format!("    {}", val(FormField::Health, &form.health))),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}:", FormField::Type.label()),
                field_style(FormField::Type),
            )),
            Line::from(format!(
                "    {}",
                val(FormField::Type, &form.pokemon_type)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Tab/↑↓ change field · Enter save · Esc cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let mut all_lines = lines;
        if let Some(err) = &form.error {
            all_lines.push(Line::from(""));
            all_lines.push(Line::from(Span::styled(
                format!("Error: {err}"),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
        }

        let para = Paragraph::new(all_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(para, area);
    }

    fn draw_message_popup(&self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(60, 30, area);
        frame.render_widget(Clear, popup);
        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from(self.message.as_str()),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to continue",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Notice ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(para, popup);
    }
}

// ---------------------------------------------------------------------------
// Audio + main loop
// ---------------------------------------------------------------------------

/// Keeps rodio's output stream alive for the whole process. Dropping the stream
/// stops playback, so this must outlive the game loop (do not call detach and exit).
struct BackgroundMusic {
    _stream: rodio::OutputStream,
    _sink: rodio::Sink,
}

impl BackgroundMusic {
    /// Start looping/background track if possible. Returns `None` on failure so
    /// the game can still run without audio hardware.
    fn try_start() -> Option<Self> {
        use std::fs::File;
        use std::io::BufReader;

        let (stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Warning: no audio output device ({e}); continuing without music.");
                return None;
            }
        };

        let sink = match rodio::Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not open audio sink ({e}); continuing without music.");
                return None;
            }
        };

        let song_file = match File::open("resources/track.mp3") {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Warning: could not open resources/track.mp3 ({e}); continuing without music."
                );
                return None;
            }
        };

        let source = match rodio::Decoder::new(BufReader::new(song_file)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not decode track.mp3 ({e}); continuing without music.");
                return None;
            }
        };

        sink.append(source);
        // Keep stream + sink owned by this struct until main exits.
        Some(Self {
            _stream: stream,
            _sink: sink,
        })
    }
}

fn run(mut terminal: DefaultTerminal, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| app.draw(frame))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // Hold this for the entire process lifetime so music keeps playing.
    let _music = BackgroundMusic::try_start();

    let terminal = ratatui::init();
    let app = App::new();
    let result = run(terminal, app);
    ratatui::restore();
    result
}
