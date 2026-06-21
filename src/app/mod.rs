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

mod audio;
mod input;
mod ui;

pub use audio::BackgroundMusic;
pub use input::handle_key;
pub use ui::draw;

use rand::Rng;
use ratatui::widgets::ListState;

use pokemon_text_game::extra::{clamp_index, move_selection, parse_i64, parse_moves};
use pokemon_text_game::{
    read_team_from_file, roll_block, roll_critical_chance, write_team_to_file, Pokemon,
    PokemonsList, TEAM_PATH,
};

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
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
pub enum SelectSlot {
    VsComputerPlayer,
    VsHumanP1,
    VsHumanP2,
}

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
pub enum PokedexAction {
    Edit,
    Delete,
}

impl PokedexAction {
    pub fn title(self) -> &'static str {
        match self {
            Self::Edit => " Select Pokemon to edit ",
            Self::Delete => " Select Pokemon to delete ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
    CreatePokemon,
    EditPokemon { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Name,
    Moves,
    Health,
    Type,
}

impl FormField {
    pub const ALL: [Self; 4] = [Self::Name, Self::Moves, Self::Health, Self::Type];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&f| f == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|&f| f == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Moves => "Moves (space-separated)",
            Self::Health => "Health",
            Self::Type => "Type (number)",
        }
    }
}

pub struct BattleState {
    pub player1: Pokemon,
    pub player2: Pokemon,
    pub player1_max_hp: i64,
    pub player2_max_hp: i64,
    pub vs_computer: bool,
    pub player1_turn: bool,
    pub move_cursor: usize,
    pub log: Vec<String>,
    pub winner_text: Option<String>,
}

impl BattleState {
    fn attacker_is_p1(&self) -> bool {
        self.player1_turn
    }

    pub fn attacker_moves(&self) -> &[String] {
        if self.player1_turn {
            &self.player1.moves_name
        } else {
            &self.player2.moves_name
        }
    }

    fn attacker_name(&self) -> &str {
        if self.player1_turn {
            &self.player1.name
        } else {
            &self.player2.name
        }
    }

    fn defender_name(&self) -> &str {
        if self.player1_turn {
            &self.player2.name
        } else {
            &self.player1.name
        }
    }

    pub fn side_label(&self, is_p1: bool) -> &'static str {
        match (is_p1, self.vs_computer) {
            (true, true) => "You",
            (true, false) => "Player 1",
            (false, true) => "Computer",
            (false, false) => "Player 2",
        }
    }

    pub fn turn_prompt(&self) -> &'static str {
        match (self.player1_turn, self.vs_computer) {
            (true, true) => "Your turn — choose a move",
            (true, false) => "Player 1's turn — choose a move",
            (false, true) => "Computer is attacking...",
            (false, false) => "Player 2's turn — choose a move",
        }
    }
}

pub struct FormState {
    pub kind: FormKind,
    pub field: FormField,
    pub name: String,
    pub moves: String,
    pub health: String,
    pub pokemon_type: String,
    pub error: Option<String>,
}

impl FormState {
    fn create() -> Self {
        Self {
            kind: FormKind::CreatePokemon,
            field: FormField::Name,
            name: String::new(),
            moves: String::new(),
            health: "400".into(),
            pokemon_type: "1".into(),
            error: None,
        }
    }

    fn edit(index: usize, p: &Pokemon) -> Self {
        Self {
            kind: FormKind::EditPokemon { index },
            field: FormField::Name,
            name: p.name.clone(),
            moves: p.moves_name.join(" "),
            health: p.health.to_string(),
            pokemon_type: p.pokemon_type.to_string(),
            error: None,
        }
    }

    pub fn field_buf_mut(&mut self) -> &mut String {
        match self.field {
            FormField::Name => &mut self.name,
            FormField::Moves => &mut self.moves,
            FormField::Health => &mut self.health,
            FormField::Type => &mut self.pokemon_type,
        }
    }

    pub fn field_value(&self, field: FormField) -> &str {
        match field {
            FormField::Name => &self.name,
            FormField::Moves => &self.moves,
            FormField::Health => &self.health,
            FormField::Type => &self.pokemon_type,
        }
    }

    fn set_error(&mut self, msg: impl Into<String>) {
        self.error = Some(msg.into());
    }
}

pub struct App {
    pub screen: Screen,
    pub team: PokemonsList,
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
}

impl App {
    pub fn new() -> Self {
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
        };

        if let Some(err) = load_error {
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

    pub fn go_main_menu(&mut self) {
        self.menu_index = 0;
        self.screen = Screen::MainMenu;
    }

    pub fn go_play_mode(&mut self) {
        self.menu_index = 0;
        self.screen = Screen::PlayMode;
    }

    pub fn go_pokedex_menu(&mut self) {
        self.menu_index = 0;
        self.screen = Screen::PokedexMenu;
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn ensure_list_selection(&mut self) {
        let len = self.team.pokeball.len();
        if len == 0 {
            self.list_state.select(None);
        } else {
            let i = self.list_state.selected().unwrap_or(0);
            self.list_state.select(Some(clamp_index(i, len)));
        }
    }

    pub fn move_list(&mut self, delta: isize) {
        let len = self.team.pokeball.len();
        if len == 0 {
            return;
        }
        let cur = self.list_state.selected().unwrap_or(0);
        self.list_state
            .select(Some(move_selection(cur, delta, len)));
    }

    pub fn move_menu(&mut self, delta: isize, items: usize) {
        self.menu_index = move_selection(self.menu_index, delta, items);
    }

    pub fn open_pokedex_list(&mut self, action: PokedexAction) {
        if self.team.pokeball.is_empty() {
            let verb = match action {
                PokedexAction::Edit => "edit",
                PokedexAction::Delete => "delete",
            };
            self.status = format!("No Pokemon to {verb}.");
            return;
        }
        self.ensure_list_selection();
        self.screen = Screen::PokedexList { action };
    }

    pub fn start_form_create(&mut self) {
        self.form = Some(FormState::create());
        self.screen = Screen::FormInput;
    }

    pub fn start_form_edit(&mut self, index: usize) {
        let p = &self.team.pokeball[index];
        self.form = Some(FormState::edit(index, p));
        self.screen = Screen::FormInput;
    }

    pub fn submit_form(&mut self) {
        let Some(form) = self.form.as_ref() else {
            return;
        };

        let name = form.name.trim().to_string();
        if name.is_empty() {
            if let Some(f) = self.form.as_mut() {
                f.set_error("Name cannot be empty.");
            }
            return;
        }

        let moves = parse_moves(&form.moves);
        if moves.is_empty() {
            if let Some(f) = self.form.as_mut() {
                f.set_error("Enter at least one move (space-separated).");
            }
            return;
        }

        let health = match parse_i64(&form.health) {
            Ok(h) if h > 0 => h,
            Ok(_) => {
                if let Some(f) = self.form.as_mut() {
                    f.set_error("Health must be a positive number.");
                }
                return;
            }
            Err(e) => {
                if let Some(f) = self.form.as_mut() {
                    f.set_error(e);
                }
                return;
            }
        };

        let pokemon_type = match parse_i64(&form.pokemon_type) {
            Ok(t) => t,
            Err(e) => {
                if let Some(f) = self.form.as_mut() {
                    f.set_error(e);
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

    pub fn delete_selected(&mut self) {
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

    pub fn begin_pokemon_select(&mut self, slot: SelectSlot) {
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

    pub fn confirm_pokemon_select(&mut self) {
        let Some(slot) = self.pending_select else {
            return;
        };
        let Some(idx) = self.selected_index() else {
            return;
        };
        if idx >= self.team.pokeball.len() {
            return;
        }

        let mut rng = rand::thread_rng();
        match slot {
            SelectSlot::VsComputerPlayer => {
                let player = self.team.pokeball[idx].clone();
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
                let player_first = rng.gen_range(0..2) == 0;
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

        if !player1_first && vs_computer {
            self.resolve_cpu_turn();
        }
    }

    pub fn battle_move_cursor(&mut self, delta: isize) {
        let Some(b) = &self.battle else {
            return;
        };
        let len = b.attacker_moves().len();
        let cur = b.move_cursor;
        if let Some(b) = &mut self.battle {
            b.move_cursor = move_selection(cur, delta, len);
        }
    }

    /// Shared attack resolution for player and CPU turns.
    fn execute_attack(&mut self, move_index: Option<usize>, cpu_prefix: bool) {
        let Some(b) = &mut self.battle else {
            return;
        };
        if b.winner_text.is_some() {
            return;
        }

        let attacker_is_p1 = b.attacker_is_p1();
        let moves = b.attacker_moves().to_vec();
        if moves.is_empty() {
            let who = if cpu_prefix { "CPU" } else { "Attacker" };
            b.log.push(format!("{who} has no moves!"));
            if cpu_prefix {
                b.player1_turn = true;
            }
            return;
        }

        let mi = move_index
            .unwrap_or(b.move_cursor)
            .min(moves.len().saturating_sub(1));
        let move_name = &moves[mi];
        let attacker_name = b.attacker_name().to_string();
        let defender_name = b.defender_name().to_string();

        let lead = if cpu_prefix {
            format!("CPU's {attacker_name} uses {move_name}!")
        } else {
            format!("{attacker_name} uses {move_name}!")
        };
        b.log.push(lead);

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

        if let Some(b) = &mut self.battle {
            b.player1_turn = !b.player1_turn;
            b.move_cursor = 0;
        }

        // After player turn vs computer, let CPU act.
        if let Some(b) = &self.battle {
            if !cpu_prefix && b.vs_computer && !b.player1_turn && b.winner_text.is_none() {
                self.resolve_cpu_turn();
            }
        }
    }

    pub fn player_attack(&mut self) {
        self.execute_attack(None, false);
    }

    fn resolve_cpu_turn(&mut self) {
        let Some(b) = &self.battle else {
            return;
        };
        if b.winner_text.is_some() || b.player1_turn {
            return;
        }
        let moves_len = b.player2.moves_name.len();
        if moves_len == 0 {
            self.execute_attack(Some(0), true);
            return;
        }
        let mi = rand::thread_rng().gen_range(0..moves_len);
        self.execute_attack(Some(mi), true);
    }

    fn check_battle_over(&mut self) -> bool {
        let Some(b) = &mut self.battle else {
            return false;
        };

        let winner = if b.player1.is_fainted() {
            Some(if b.vs_computer {
                format!("Computer wins with {}!", b.player2.name)
            } else {
                format!("Player 2 wins with {}!", b.player2.name)
            })
        } else if b.player2.is_fainted() {
            Some(if b.vs_computer {
                format!("You win with {}!", b.player1.name)
            } else {
                format!("Player 1 wins with {}!", b.player1.name)
            })
        } else {
            None
        };

        let Some(winner) = winner else {
            return false;
        };
        b.winner_text = Some(winner.clone());
        b.log.push(winner);
        self.screen = Screen::BattleOver;
        true
    }

    pub fn end_battle_to_menu(&mut self) {
        self.battle = None;
        self.go_main_menu();
    }
}
