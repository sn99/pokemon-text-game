use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{App, PokedexAction, Screen, SelectSlot};
use pokemon_text_game::pokemon::all_species;
use pokemon_text_game::world::{default_gyms, elite_four, mart_catalog};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press { return; }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true; return;
    }
    match app.screen {
        Screen::MainMenu => key_main_menu(app, key),
        Screen::PlayMode => key_play_mode(app, key),
        Screen::SelectPokemon { .. } => key_select_pokemon(app, key),
        Screen::Battle => key_battle(app, key),
        Screen::BattleOver => key_battle_over(app, key),
        Screen::PokedexMenu => key_pokedex_menu(app, key),
        Screen::PokedexList { action } => key_pokedex_list(app, key, action),
        Screen::SpeciesDex => key_species_dex(app, key),
        Screen::FormInput => key_form(app, key),
        Screen::Message => key_message(app, key),
        Screen::AdventureHub => key_adventure(app, key),
        Screen::PartyView => key_party(app, key),
        Screen::InventoryView => key_inventory(app, key),
        Screen::SettingsView => key_settings(app, key),
        Screen::StarterSelect => key_starter(app, key),
        Screen::Help => key_help(app, key),
        Screen::RouteSelect => key_routes(app, key),
        Screen::GymSelect => key_gyms(app, key),
        Screen::Achievements => key_help(app, key),
        Screen::BoxStorage => key_box(app, key),
        Screen::TypeChart => key_help(app, key),
        Screen::Mart => key_mart(app, key),
        Screen::EliteFour => key_elite(app, key),
        Screen::NicknameInput => key_nickname(app, key),
    }
}

fn key_main_menu(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 7;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.go_adventure(),
            1 => app.go_play_mode(),
            2 => app.go_pokedex_menu(),
            3 => app.go_species_dex(),
            4 => app.go_type_chart(),
            5 => app.go_help(),
            _ => app.should_quit = true,
        },
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('1') => app.go_adventure(),
        KeyCode::Char('2') => app.go_play_mode(),
        KeyCode::Char('3') => app.go_pokedex_menu(),
        KeyCode::Char('4') => app.go_species_dex(),
        _ => {}
    }
}

fn key_play_mode(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 3;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Esc | KeyCode::Char('q') => app.go_main_menu(),
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.begin_pokemon_select(SelectSlot::VsComputerPlayer),
            1 => app.begin_pokemon_select(SelectSlot::VsHumanP1),
            _ => app.go_main_menu(),
        },
        _ => {}
    }
}

fn key_select_pokemon(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_list(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_list(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.confirm_pokemon_select(),
        KeyCode::Esc | KeyCode::Char('q') => { app.pending_select = None; app.go_play_mode(); }
        _ => {}
    }
}

fn key_battle(app: &mut App, key: KeyEvent) {
    let Some(b) = &app.battle else { return; };
    if b.vs_computer && !b.player1_turn { return; }

    // Battle command menu when not in move pick (advanced)
    if b.mode == super::BattleMode::Advanced && !b.in_move_pick {
        match key.code {
            KeyCode::Char('1') | KeyCode::Char('f') => {
                if let Some(b) = &mut app.battle { b.in_move_pick = true; b.move_cursor = 0; }
            }
            KeyCode::Char('2') | KeyCode::Char('b') => app.try_catch(),
            KeyCode::Char('3') | KeyCode::Char('i') => app.battle_use_potion(),
            KeyCode::Char('4') | KeyCode::Char('p') => app.battle_switch_party(),
            KeyCode::Char('5') | KeyCode::Char('r') => app.try_flee(),
            KeyCode::Char('s') => app.toggle_sprites(),
            KeyCode::Esc | KeyCode::Char('q') => app.end_battle_to_menu(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(b) = &mut app.battle { b.in_move_pick = true; }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.battle_move_cursor(-1),
        KeyCode::Down | KeyCode::Char('j') => app.battle_move_cursor(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.player_attack(),
        KeyCode::Char('b') => app.try_catch(),
        KeyCode::Char('r') => app.try_flee(),
        KeyCode::Char('s') => app.toggle_sprites(),
        KeyCode::Esc | KeyCode::Char('q') => {
            if let Some(b) = &mut app.battle {
                if b.mode == super::BattleMode::Advanced && b.in_move_pick {
                    b.in_move_pick = false;
                    return;
                }
            }
            app.end_battle_to_menu();
        }
        _ => {}
    }
}

fn key_battle_over(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q')) {
        app.end_battle_to_menu();
    }
}

fn key_pokedex_menu(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 5;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Esc | KeyCode::Char('q') => app.go_main_menu(),
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.start_form_create(),
            1 => app.open_pokedex_list(PokedexAction::Edit),
            2 => app.open_pokedex_list(PokedexAction::Delete),
            3 => app.go_species_dex(),
            _ => app.go_main_menu(),
        },
        _ => {}
    }
}

fn key_pokedex_list(app: &mut App, key: KeyEvent, action: PokedexAction) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_list(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_list(1),
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::PokedexMenu,
        KeyCode::Enter | KeyCode::Char(' ') => match action {
            PokedexAction::Edit => { if let Some(i) = app.selected_index() { app.start_form_edit(i); } }
            PokedexAction::Delete => app.delete_selected(),
        },
        _ => {}
    }
}

fn key_species_dex(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_species(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_species(1),
        KeyCode::PageUp => { for _ in 0..10 { app.move_species(-1); } }
        KeyCode::PageDown => { for _ in 0..10 { app.move_species(1); } }
        KeyCode::Home => {
            let idxs = app.filtered_species_indices();
            if let Some(&i) = idxs.first() { app.species_index = i; }
        }
        KeyCode::End => {
            let idxs = app.filtered_species_indices();
            if let Some(&i) = idxs.last() { app.species_index = i; }
        }
        KeyCode::Char('f') => app.cycle_dex_filter(),
        KeyCode::Char('b') | KeyCode::Enter => app.battle_selected_species(),
        KeyCode::Char('r') => app.buy_dex_mon(),
        // Cheat: press 'g' to jump to numeric id in search buffer
        KeyCode::Char('g') => {
            if let Ok(id) = app.dex_search.parse::<u16>() {
                app.jump_dex_to_id(id);
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            if matches!(app.message_return, Screen::AdventureHub) || app.save.party.iter().any(|_| true) {
                // Prefer adventure if we came from there
            }
            app.go_main_menu();
        }
        KeyCode::Backspace => { app.dex_search.pop(); }
        KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == '-' => {
            app.dex_search.push(c);
            let idxs = app.filtered_species_indices();
            if let Some(&i) = idxs.first() { app.species_index = i; }
        }
        _ => {}
    }
}

fn key_form(app: &mut App, key: KeyEvent) {
    let Some(form) = app.form.as_mut() else { return; };
    match key.code {
        KeyCode::Esc => { app.form = None; app.screen = Screen::PokedexMenu; }
        KeyCode::Tab | KeyCode::Down => { form.field = form.field.next(); form.error = None; }
        KeyCode::BackTab | KeyCode::Up => { form.field = form.field.prev(); form.error = None; }
        KeyCode::Enter => app.submit_form(),
        KeyCode::Backspace => { form.error = None; form.field_buf_mut().pop(); }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            form.error = None; form.field_buf_mut().push(c);
        }
        _ => {}
    }
}

fn key_message(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q')) {
        app.screen = app.message_return;
        app.message.clear();
        if app.screen == Screen::AdventureHub { app.menu_index = 0; }
    }
}

fn key_adventure(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 16;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Esc | KeyCode::Char('q') => app.go_main_menu(),
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.start_wild_battle(),
            1 => app.start_water_battle(),
            2 => app.start_route_trainer(),
            3 => app.start_rival_battle(),
            4 => app.go_routes(),
            5 => app.go_gyms(),
            6 => app.go_elite(),
            7 => { app.menu_index = 0; app.screen = Screen::PartyView; }
            8 => { app.menu_index = 0; app.screen = Screen::InventoryView; }
            9 => app.go_mart(),
            10 => app.go_box(),
            11 => app.heal_at_center(),
            12 => app.go_species_dex(),
            13 => app.go_achievements(),
            14 => app.persist_save(),
            15 => app.go_settings(),
            _ => app.go_main_menu(),
        },
        _ => {}
    }
}

fn key_party(app: &mut App, key: KeyEvent) {
    let len = app.save.party.len().max(1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, len),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, len),
        KeyCode::Char('d') => {
            match app.save.try_deposit_to_box(app.menu_index) {
                Ok(m) => { app.status = m; app.persist_save(); }
                Err(e) => app.status = e,
            }
        }
        KeyCode::Char('n') => app.start_nickname(),
        KeyCode::Char('t') => app.tutor_selected_party(),
        KeyCode::Esc | KeyCode::Char('q') => { app.menu_index = 0; app.screen = Screen::AdventureHub; }
        _ => {}
    }
}

fn key_inventory(app: &mut App, key: KeyEvent) {
    let len = app.save.inventory.len().max(1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, len),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, len),
        KeyCode::Enter | KeyCode::Char(' ') => {
            if app.save.inventory.is_empty() || app.save.party.is_empty() { app.status = "Nothing to use.".into(); return; }
            let inv_i = app.menu_index.min(app.save.inventory.len() - 1);
            match app.save.use_item_on_party(inv_i, 0) {
                Ok(msg) => { app.status = msg; app.persist_save(); }
                Err(e) => app.status = e,
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => { app.menu_index = 0; app.screen = Screen::AdventureHub; }
        _ => {}
    }
}

fn key_settings(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 6;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Left | KeyCode::Char('-') => {
            if app.menu_index == 3 { app.adjust_volume(-0.05); }
        }
        KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.menu_index == 3 { app.adjust_volume(0.05); }
        }
        KeyCode::Enter | KeyCode::Char(' ') => match app.menu_index {
            0 => app.toggle_sprites(),
            1 => {
                app.save.settings.music_enabled = !app.save.settings.music_enabled;
                app.pending_audio_enabled = Some(app.save.settings.music_enabled);
                app.status = format!("Music: {}", if app.save.settings.music_enabled { "ON" } else { "OFF" });
            }
            2 => {
                app.save.settings.color_blind_mode = !app.save.settings.color_blind_mode;
                app.status = format!("Color-blind assist: {}", if app.save.settings.color_blind_mode { "ON" } else { "OFF" });
            }
            3 => app.adjust_volume(0.05),
            4 => app.export_save(),
            _ => { app.persist_save(); app.screen = Screen::AdventureHub; }
        },
        KeyCode::Esc | KeyCode::Char('q') => { app.menu_index = 0; app.screen = Screen::AdventureHub; }
        _ => {}
    }
}

fn key_starter(app: &mut App, key: KeyEvent) {
    const ITEMS: usize = 3;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, ITEMS),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, ITEMS),
        KeyCode::Enter | KeyCode::Char(' ') => app.pick_starter(app.menu_index),
        KeyCode::Char('1') => app.pick_starter(0),
        KeyCode::Char('2') => app.pick_starter(1),
        KeyCode::Char('3') => app.pick_starter(2),
        KeyCode::Esc | KeyCode::Char('q') => app.go_main_menu(),
        _ => {}
    }
}

fn key_help(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q')) {
        if app.screen == Screen::TypeChart || app.screen == Screen::Achievements {
            app.go_adventure();
        } else {
            app.go_main_menu();
        }
    }
}

fn key_routes(app: &mut App, key: KeyEvent) {
    let n = pokemon_text_game::world::default_routes().len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, n),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, n),
        KeyCode::Enter | KeyCode::Char(' ') => {
            let id = (app.menu_index as u8) + 1;
            app.start_route_wild(id);
        }
        KeyCode::Char('w') => {
            let id = (app.menu_index as u8) + 1;
            app.save.current_route = id;
            app.start_water_battle();
        }
        KeyCode::Char('t') => {
            let id = (app.menu_index as u8) + 1;
            app.save.current_route = id;
            app.start_route_trainer();
        }
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::AdventureHub,
        _ => {}
    }
}

fn key_mart(app: &mut App, key: KeyEvent) {
    let n = mart_catalog().len().max(1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, n),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, n),
        KeyCode::Enter | KeyCode::Char(' ') => app.buy_mart_item(),
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::AdventureHub,
        _ => {}
    }
}

fn key_elite(app: &mut App, key: KeyEvent) {
    let n = elite_four().len().max(1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, n),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, n),
        KeyCode::Enter | KeyCode::Char(' ') => app.start_elite_battle(app.menu_index),
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::AdventureHub,
        _ => {}
    }
}

fn key_nickname(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.screen = Screen::PartyView,
        KeyCode::Enter => app.submit_nickname(),
        KeyCode::Backspace => { app.nickname_buf.pop(); }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.nickname_buf.len() < 12 { app.nickname_buf.push(c); }
        }
        _ => {}
    }
}

fn key_gyms(app: &mut App, key: KeyEvent) {
    let n = default_gyms().len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, n),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, n),
        KeyCode::Enter | KeyCode::Char(' ') => app.start_gym_battle(app.menu_index),
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::AdventureHub,
        _ => {}
    }
}

fn key_box(app: &mut App, key: KeyEvent) {
    let len = app.save.box_storage.len().max(1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.move_menu(-1, len),
        KeyCode::Down | KeyCode::Char('j') => app.move_menu(1, len),
        KeyCode::Enter | KeyCode::Char('w') => {
            match app.save.try_withdraw_from_box(app.menu_index) {
                Ok(m) => { app.status = m; app.persist_save(); }
                Err(e) => app.status = e,
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => app.screen = Screen::AdventureHub,
        _ => {}
    }
}
