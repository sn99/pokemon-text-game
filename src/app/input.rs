use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{App, PokedexAction, Screen, SelectSlot};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    match app.screen {
        Screen::MainMenu => key_main_menu(app, key),
        Screen::PlayMode => key_play_mode(app, key),
        Screen::SelectPokemon { .. } => key_select_pokemon(app, key),
        Screen::Battle => key_battle(app, key),
        Screen::BattleOver => key_battle_over(app, key),
        Screen::PokedexMenu => key_pokedex_menu(app, key),
        Screen::PokedexList { action } => key_pokedex_list(app, key, action),
        Screen::FormInput => key_form(app, key),
        Screen::Message => key_message(app, key),
    }
}

fn key_main_menu(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 3;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.go_play_mode(),
            1 => app.go_pokedex_menu(),
            _ => app.should_quit = true,
        },
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('1') => app.go_play_mode(),
        KeyCode::Char('2') => app.go_pokedex_menu(),
        KeyCode::Char('3') => app.should_quit = true,
        _ => {}
    }
}

fn key_play_mode(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 3;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('3') => app.go_main_menu(),
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('1') | KeyCode::Char('2') => {
            let choice = match key.code {
                KeyCode::Char('1') => 0,
                KeyCode::Char('2') => 1,
                _ => app.menu_index,
            };
            match choice {
                0 => app.begin_pokemon_select(SelectSlot::VsComputerPlayer),
                1 => app.begin_pokemon_select(SelectSlot::VsHumanP1),
                _ => app.go_main_menu(),
            }
        }
        _ => {}
    }
}

fn key_select_pokemon(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_list(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_list(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.confirm_pokemon_select(),
        KeyCode::Esc | KeyCode::Char('q') => {
            app.pending_select = None;
            app.selected_p1_index = None;
            app.go_play_mode();
        }
        _ => {}
    }
}

fn key_battle(app: &mut App, key: KeyEvent) {
    let Some(b) = &app.battle else {
        return;
    };
    if b.vs_computer && !b.player1_turn {
        return;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.battle_move_cursor(-1),
        KeyCode::Down | KeyCode::Char('j') => app.battle_move_cursor(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.player_attack(),
        KeyCode::Esc | KeyCode::Char('q') => app.end_battle_to_menu(),
        _ => {}
    }
}

fn key_battle_over(app: &mut App, key: KeyEvent) {
    if matches!(
        key.code,
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q')
    ) {
        app.end_battle_to_menu();
    }
}

fn key_pokedex_menu(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 4;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('4') => app.go_main_menu(),
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.start_form_create(),
            1 => app.open_pokedex_list(PokedexAction::Edit),
            2 => app.open_pokedex_list(PokedexAction::Delete),
            _ => app.go_main_menu(),
        },
        KeyCode::Char('1') => app.start_form_create(),
        KeyCode::Char('2') => app.open_pokedex_list(PokedexAction::Edit),
        KeyCode::Char('3') => app.open_pokedex_list(PokedexAction::Delete),
        _ => {}
    }
}

fn key_pokedex_list(app: &mut App, key: KeyEvent, action: PokedexAction) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_list(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_list(1),
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::PokedexMenu,
        KeyCode::Enter | KeyCode::Char(' ') => match action {
            PokedexAction::Edit => {
                if let Some(i) = app.selected_index() {
                    app.start_form_edit(i);
                }
            }
            PokedexAction::Delete => app.delete_selected(),
        },
        _ => {}
    }
}

fn key_form(app: &mut App, key: KeyEvent) {
    let Some(form) = app.form.as_mut() else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.form = None;
            app.screen = Screen::PokedexMenu;
        }
        KeyCode::Tab | KeyCode::Down => {
            form.field = form.field.next();
            form.error = None;
        }
        KeyCode::BackTab | KeyCode::Up => {
            form.field = form.field.prev();
            form.error = None;
        }
        KeyCode::Enter => app.submit_form(),
        KeyCode::Backspace => {
            form.error = None;
            form.field_buf_mut().pop();
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            form.error = None;
            form.field_buf_mut().push(c);
        }
        _ => {}
    }
}

fn key_message(app: &mut App, key: KeyEvent) {
    if matches!(
        key.code,
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q')
    ) {
        app.screen = app.message_return;
        app.message.clear();
    }
}
