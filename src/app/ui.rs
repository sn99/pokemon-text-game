use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use super::{App, BattleMode, DexFilter, FormField, FormKind, Screen};
use pokemon_text_game::ascii::color_sprite_for_species;
use pokemon_text_game::pokemon::{all_species, db_stats, ElementType, Pokemon};
use pokemon_text_game::world::{
    daily_challenge_for_day, default_gyms, default_routes, elite_four, mart_catalog, species_flavor,
};

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2),
    ]).split(area);
    Layout::default().direction(Direction::Horizontal).constraints([
        Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2),
    ]).split(popup_layout[1])[1]
}

fn highlight_style() -> Style { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) }

fn menu_items<'a>(labels: &[&'a str], selected: usize) -> Vec<ListItem<'a>> {
    labels.iter().enumerate().map(|(i, label)| {
        let prefix = if i == selected { "> " } else { "  " };
        let style = if i == selected { highlight_style() } else { Style::default().fg(Color::White) };
        ListItem::new(Line::from(Span::styled(format!("{prefix}{}. {}", i + 1, label), style)))
    }).collect()
}

fn help_line(text: &str) -> Paragraph<'_> {
    Paragraph::new(text).style(Style::default().fg(Color::DarkGray)).alignment(Alignment::Center)
}

fn bordered_list<'a>(title: &'a str, border: Color, items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(items).block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(border)))
}

fn pokemon_list_items(pokemons: &[Pokemon], detailed: bool) -> Vec<ListItem<'_>> {
    pokemons.iter().enumerate().map(|(i, p)| {
        let line = if detailed {
            format!("{}. {}  HP:{}  type:{}  moves:{}", i + 1, p.name, p.health, p.pokemon_type, p.moves_name.join(", "))
        } else {
            format!("{}. {}  HP:{}  moves:{}", i + 1, p.name, p.health, p.moves_name.join(", "))
        };
        ListItem::new(line)
    }).collect()
}

pub fn draw(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(3), Constraint::Min(5), Constraint::Length(2),
    ]).split(area);

    let (n_sp, n_mv, _, _) = db_stats();
    let day = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() / 86400) as u32;
    let daily = daily_challenge_for_day(day);
    let repel = if app.save.repel_steps > 0 { format!(" · repel{}", app.save.repel_steps) } else { String::new() };
    let subtitle = format!(
        " v2.2 · {} · ${} · {}W/{}L · {}★ · dex {}/{} · {}sp/{}mv · ⏱{}{repel} ",
        app.save.player_name, app.save.money, app.save.stats.battles_won, app.save.stats.battles_lost,
        app.save.badge_count(), app.save.dex.seen_count(), app.save.dex.caught_count(), n_sp, n_mv,
        app.save.play_time_display()
    );
    let title = Paragraph::new(vec![
        Line::from(Span::styled(" ⚔ Pokemon Text Game ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(subtitle, Style::default().fg(Color::DarkGray))),
    ]).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    match app.screen {
        Screen::MainMenu => draw_simple_menu(frame, chunks[1], " Main Menu ", Color::Green, &[
            "Adventure mode", "Quick battle (classic team)", "Team editor",
            "Species database (battle any!)", "Type chart", "Help", "Exit",
        ], app.menu_index),
        Screen::PlayMode => draw_simple_menu(frame, chunks[1], " Play Mode ", Color::Green, &["Vs Computer", "Vs Human", "Back"], app.menu_index),
        Screen::SelectPokemon { slot } => draw_pokemon_picker(app, frame, chunks[1], slot.title()),
        Screen::Battle | Screen::BattleOver => draw_battle(app, frame, chunks[1]),
        Screen::PokedexMenu => draw_simple_menu(frame, chunks[1], " Team Editor ", Color::LightBlue, &[
            "Create", "Edit", "Delete", "Species dex", "Back",
        ], app.menu_index),
        Screen::PokedexList { action } => draw_pokemon_picker_detailed(app, frame, chunks[1], action.title()),
        Screen::SpeciesDex => draw_species_dex(app, frame, chunks[1], &daily.description),
        Screen::FormInput => draw_form(app, frame, chunks[1]),
        Screen::Message => draw_message_popup(app, frame, area),
        Screen::AdventureHub => draw_simple_menu(frame, chunks[1], " Adventure Hub ", Color::Magenta, &[
            "Wild battle (grass)", "Fish / water battle", "Route trainer", "Rival battle",
            "Choose route", "Gym challenge", "Elite Four",
            "Party", "Bag", "Poké Mart", "PC box", "Pokemon Center",
            "Species dex", "Achievements", "Save", "Settings",
        ], app.menu_index),
        Screen::PartyView => draw_party(app, frame, chunks[1]),
        Screen::InventoryView => draw_inventory(app, frame, chunks[1]),
        Screen::SettingsView => draw_settings(app, frame, chunks[1]),
        Screen::StarterSelect => draw_simple_menu(frame, chunks[1], " Choose starter! ", Color::Yellow, &[
            "Bulbasaur + Pikachu", "Charmander + Pikachu", "Squirtle + Pikachu",
        ], app.menu_index),
        Screen::Help => draw_help(frame, chunks[1]),
        Screen::RouteSelect => draw_routes(app, frame, chunks[1]),
        Screen::GymSelect => draw_gyms(app, frame, chunks[1]),
        Screen::Achievements => draw_achievements(app, frame, chunks[1]),
        Screen::BoxStorage => draw_box(app, frame, chunks[1]),
        Screen::TypeChart => draw_type_chart(frame, chunks[1]),
        Screen::Mart => draw_mart(app, frame, chunks[1]),
        Screen::EliteFour => draw_elite(app, frame, chunks[1]),
        Screen::NicknameInput => draw_nickname(app, frame, chunks[1]),
    }

    let status_text = if !app.status.is_empty() { app.status.as_str() } else {
        "↑↓ · Enter · q · Dex:b/r/f · Battle:f/b/i/p/r · Party:n/t · Routes:w/t"
    };
    frame.render_widget(help_line(status_text), chunks[2]);
}

fn draw_simple_menu(frame: &mut Frame, area: Rect, title: &str, border: Color, labels: &[&str], selected: usize) {
    frame.render_widget(bordered_list(title, border, menu_items(labels, selected)), area);
}

fn draw_settings(app: &App, frame: &mut Frame, area: Rect) {
    let vol = format!("Volume: {:.0}%  (−/+)", app.save.settings.music_volume * 100.0);
    let labels = [
        if app.show_sprites { "Color sprites: ON" } else { "Color sprites: OFF" },
        if app.save.settings.music_enabled { "Music: ON" } else { "Music: OFF" },
        if app.save.settings.color_blind_mode { "Color-blind assist: ON" } else { "Color-blind assist: OFF" },
        vol.as_str(),
        "Export save copy",
        "Save & back",
    ];
    frame.render_widget(bordered_list(" Settings ", Color::Gray, menu_items(&labels, app.menu_index)), area);
}

fn draw_pokemon_picker(app: &mut App, frame: &mut Frame, area: Rect, title: &str) {
    let items = pokemon_list_items(&app.team.pokeball, false);
    let list = bordered_list(title, Color::Magenta, items).highlight_style(highlight_style()).highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_pokemon_picker_detailed(app: &mut App, frame: &mut Frame, area: Rect, title: &str) {
    let items = pokemon_list_items(&app.team.pokeball, true);
    let list = bordered_list(title, Color::Magenta, items).highlight_style(highlight_style()).highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_hp_gauge(frame: &mut Frame, area: Rect, label: &str, pokemon: &Pokemon, max_hp: i64, color: Color, extra: &str) {
    let title = if extra.is_empty() { format!(" {label}: {} ", pokemon.name) } else { format!(" {label}: {} {} ", pokemon.name, extra) };
    let gauge = Gauge::default().block(Block::default().title(title)).gauge_style(Style::default().fg(color))
        .ratio(pokemon.health_ratio(max_hp)).label(format!("{} / {}", pokemon.health.max(0), max_hp));
    frame.render_widget(gauge, area);
}

fn draw_battle(app: &App, frame: &mut Frame, area: Rect) {
    let Some(b) = &app.battle else { return; };
    let has_color = !b.color_sprite_lines.is_empty() && app.show_sprites;
    let has_mono = !b.sprite_lines.is_empty() && app.show_sprites && !has_color;
    let has_sprites = has_color || has_mono;
    let sprite_h = if has_color {
        (b.color_sprite_lines.len() as u16).saturating_add(2).min(16)
    } else if has_mono {
        (b.sprite_lines.len() as u16).saturating_add(2).min(18)
    } else { 0 };
    let mut constraints = Vec::new();
    if has_sprites { constraints.push(Constraint::Length(sprite_h)); }
    constraints.extend([Constraint::Length(3), Constraint::Length(3), Constraint::Min(4), Constraint::Length(10)]);
    let rows = Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
    let mut ri = 0usize;
    if has_sprites {
        let weather_t = format!(" Battlefield · {} ", b.weather.display_name());
        if has_color {
            frame.render_widget(
                Paragraph::new(b.color_sprite_lines.clone())
                    .block(Block::default().title(weather_t).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))),
                rows[ri],
            );
        } else {
            let art: Vec<Line> = b.sprite_lines.iter().map(|s| Line::from(s.as_str())).collect();
            frame.render_widget(
                Paragraph::new(art)
                    .block(Block::default().title(weather_t).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))),
                rows[ri],
            );
        }
        ri += 1;
    }
    let p1_extra = b.inst_p1.as_ref().map(|p| {
        format!("Lv{} [{}] {} IV{}/{}{}", p.level, p.primary_type().display_name(), p.nature.display_name(), p.iv_attack, p.iv_speed, p.shiny_prefix())
    }).unwrap_or_default();
    let p2_extra = b.inst_p2.as_ref().map(|p| {
        format!("Lv{} [{}]{}{}", p.level, p.primary_type().display_name(), if p.shiny { " ✦" } else { "" }, if p.status.label().is_empty() { String::new() } else { format!(" {}", p.status.label()) })
    }).unwrap_or_default();
    draw_hp_gauge(frame, rows[ri], b.side_label(true), &b.player1, b.player1_max_hp, Color::Green, &p1_extra); ri += 1;
    draw_hp_gauge(frame, rows[ri], b.side_label(false), &b.player2, b.player2_max_hp, Color::Red, &p2_extra); ri += 1;
    let log_lines: Vec<Line> = b.log.iter().rev().take(14).rev().map(|s| Line::from(s.as_str())).collect();
    frame.render_widget(Paragraph::new(log_lines).block(Block::default().title(" Battle Log ").borders(Borders::ALL)).wrap(Wrap { trim: true }), rows[ri]); ri += 1;

    if app.screen == Screen::BattleOver {
        let win = b.winner_text.as_deref().unwrap_or("Battle over");
        frame.render_widget(Paragraph::new(vec![
            Line::from(""), Line::from(Span::styled(win, highlight_style())), Line::from(""),
            Line::from("Press Enter to continue"),
        ]).alignment(Alignment::Center).block(Block::default().title(" Result ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))), rows[ri]);
        return;
    }

    if b.mode == BattleMode::Advanced && !b.in_move_pick {
        let ball_hint = if b.can_catch { "2/b Ball" } else { "2/b (trainer — n/a)" };
        let lines = vec![
            Line::from(Span::styled(" Commands ", highlight_style())),
            Line::from("  1/f  Fight"),
            Line::from(format!("  {ball_hint}")),
            Line::from("  3/i  Item (potion)"),
            Line::from("  4/p  Switch party"),
            Line::from("  5/r  Run (wild only)"),
            Line::from("  s    Toggle sprites"),
            Line::from(b.turn_prompt()),
        ];
        frame.render_widget(Paragraph::new(lines).block(Block::default().title(" Action ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue))), rows[ri]);
        return;
    }

    let whose = b.turn_prompt();
    let moves = b.move_labels();
    let items: Vec<ListItem> = moves.iter().enumerate().map(|(i, m)| {
        let selected = i == b.move_cursor;
        let prefix = if selected { "> " } else { "  " };
        let style = if selected { highlight_style() } else { Style::default() };
        ListItem::new(Line::from(Span::styled(format!("{prefix}{}. {m}", i + 1), style)))
    }).collect();
    frame.render_widget(bordered_list(&format!(" Moves — {whose} "), Color::Blue, items), rows[ri]);
}

fn draw_form(app: &App, frame: &mut Frame, area: Rect) {
    let Some(form) = &app.form else { return; };
    let title = match form.kind { FormKind::CreatePokemon => " Create ", FormKind::EditPokemon { .. } => " Edit " };
    let field_style = |f: FormField| if form.field == f { highlight_style() } else { Style::default().fg(Color::Gray) };
    let val = |f: FormField| { let s = form.field_value(f); let d = if s.is_empty() { " " } else { s }; let c = if form.field == f { "▌" } else { "" }; format!("{d}{c}") };
    let mut lines = Vec::new();
    for field in FormField::ALL {
        lines.push(Line::from(Span::styled(format!("  {}:", field.label()), field_style(field))));
        lines.push(Line::from(format!("    {}", val(field))));
        lines.push(Line::from(""));
    }
    if let Some(err) = &form.error {
        lines.push(Line::from(Span::styled(format!("Error: {err}"), Style::default().fg(Color::Red))));
    }
    frame.render_widget(Paragraph::new(lines).block(Block::default().title(title).borders(Borders::ALL)).wrap(Wrap { trim: false }), area);
}

fn draw_message_popup(app: &App, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(vec![
        Line::from(""), Line::from(app.message.as_str()), Line::from(""),
        Line::from(Span::styled("Press Enter", Style::default().fg(Color::DarkGray))),
    ]).alignment(Alignment::Center).wrap(Wrap { trim: true }).block(Block::default().title(" Notice ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))), popup);
}

fn draw_species_dex(app: &App, frame: &mut Frame, area: Rect, daily: &str) {
    let species = all_species();
    let idxs = app.filtered_species_indices();
    let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(42), Constraint::Percentage(58)]).split(area);

    let items: Vec<ListItem> = idxs.iter().take(2000).map(|&i| {
        let s = &species[i];
        let selected = i == app.species_index;
        let prefix = if selected { "> " } else { "  " };
        let mark = if app.save.dex.caught.contains(&s.id) { "●" } else if app.save.dex.seen.contains(&s.id) { "○" } else { "·" };
        let style = if selected { highlight_style() } else { Style::default() };
        ListItem::new(Line::from(Span::styled(format!("{prefix}{mark} #{:03} {}", s.id, s.name), style)))
    }).collect();
    let filt = match app.dex_filter {
        DexFilter::All => "all", DexFilter::Seen => "seen", DexFilter::Caught => "caught", DexFilter::ByType(_) => "type",
    };
    let search = if app.dex_search.is_empty() { String::new() } else { format!(" q:{}", app.dex_search) };
    frame.render_widget(bordered_list(&format!(" Dex [{filt}]{search} · b=fight r=buy f=filter "), Color::LightBlue, items), cols[0]);

    if let Some(s) = species.get(app.species_index) {
        let sprite = color_sprite_for_species(s.id, s.primary_type());
        let mut lines: Vec<Line> = sprite.to_lines();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!("#{} {}", s.id, s.name), highlight_style())));
        lines.push(Line::from(format!("Type: {}", s.type_label())));
        lines.push(Line::from(format!("Base: HP{} Atk{} Def{} SpA{} SpD{} Spe{}", s.base_stats.hp, s.base_stats.attack, s.base_stats.defense, s.base_stats.sp_attack, s.base_stats.sp_defense, s.base_stats.speed)));
        lines.push(Line::from(format!("Catch rate: {}  Height: {}dm  Weight: {}hg", s.capture_rate, s.height_dm, s.weight_hg)));
        lines.push(Line::from(""));
        let flavor = if s.description.is_empty() { species_flavor(s.id, &s.name) } else { s.description.clone() };
        lines.push(Line::from(flavor));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Enter/b: battle · r: recruit $500 · type# then g: jump", Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(Span::styled(daily, Style::default().fg(Color::Magenta))));
        frame.render_widget(Paragraph::new(lines).block(Block::default().title(" Entry ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))).wrap(Wrap { trim: true }), cols[1]);
    }
}

fn draw_party(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app.save.party.iter().enumerate().map(|(i, p)| {
        let selected = i == app.menu_index;
        let prefix = if selected { "> " } else { "  " };
        let faint = if p.is_fainted() { " [FAINTED]" } else { "" };
        let shiny = p.shiny_prefix();
        let style = if selected { highlight_style() } else if p.is_fainted() { Style::default().fg(Color::DarkGray) } else { Style::default() };
        ListItem::new(Line::from(Span::styled(format!(
            "{prefix}{}. {shiny}{} Lv{} HP {}/{} [{}] {} IV{}/{}{faint}",
            i+1, p.display_name(), p.level, p.current_hp, p.max_hp, p.primary_type().display_name(),
            p.nature.display_name(), p.iv_attack, p.iv_speed
        ), style)))
    }).collect();
    frame.render_widget(bordered_list(" Party (d=PC · n=nickname · t=tutor) ", Color::Green, items), area);
}

fn draw_inventory(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = if app.save.inventory.is_empty() {
        vec![ListItem::new("  (empty)")]
    } else {
        app.save.inventory.iter().enumerate().map(|(i, it)| {
            let selected = i == app.menu_index;
            let prefix = if selected { "> " } else { "  " };
            let style = if selected { highlight_style() } else { Style::default() };
            ListItem::new(Line::from(Span::styled(format!("{prefix}{} x{}", it.name, it.count), style)))
        }).collect()
    };
    frame.render_widget(bordered_list(" Bag (Enter=use on lead) ", Color::Yellow, items), area);
}

fn draw_routes(app: &App, frame: &mut Frame, area: Rect) {
    let routes = default_routes();
    let items: Vec<ListItem> = routes.iter().enumerate().map(|(i, r)| {
        let selected = i == app.menu_index;
        let prefix = if selected { "> " } else { "  " };
        let cur = if r.id == app.save.current_route { " ★" } else { "" };
        let water = if r.water_pool.is_empty() { "" } else { " ~" };
        let style = if selected { highlight_style() } else { Style::default() };
        ListItem::new(Line::from(Span::styled(
            format!("{prefix}{} (Lv{}-{}) {}{water}{cur}", r.name, r.min_level, r.max_level, r.weather.display_name()),
            style,
        )))
    }).collect();
    frame.render_widget(bordered_list(" Routes — Enter grass · w water · t trainer ", Color::Green, items), area);
}

fn draw_mart(app: &App, frame: &mut Frame, area: Rect) {
    let catalog = mart_catalog();
    let items: Vec<ListItem> = catalog.iter().enumerate().map(|(i, it)| {
        let selected = i == app.menu_index;
        let prefix = if selected { "> " } else { "  " };
        let style = if selected { highlight_style() } else { Style::default() };
        ListItem::new(Line::from(Span::styled(
            format!("{prefix}{} — ${}", it.kind.display_name(), it.price),
            style,
        )))
    }).collect();
    frame.render_widget(bordered_list(&format!(" Poké Mart — you have ${} ", app.save.money), Color::Yellow, items), area);
}

fn draw_elite(app: &App, frame: &mut Frame, area: Rect) {
    let e4 = elite_four();
    let items: Vec<ListItem> = e4.iter().enumerate().map(|(i, e)| {
        let selected = i == app.menu_index;
        let prefix = if selected { "> " } else { "  " };
        let done = if (i as u8) < app.save.elite_progress { "✓" } else { " " };
        let style = if selected { highlight_style() } else if (i as u8) < app.save.elite_progress { Style::default().fg(Color::Green) } else { Style::default() };
        ListItem::new(Line::from(Span::styled(
            format!("{prefix}[{done}] {} — {} Lv{} ${}", e.name, e.specialty.display_name(), e.level, e.reward_money),
            style,
        )))
    }).collect();
    frame.render_widget(bordered_list(" Elite Four — need 8 badges ", Color::Magenta, items), area);
}

fn draw_nickname(app: &App, frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("Enter a nickname (max 12 chars):"),
        Line::from(""),
        Line::from(Span::styled(format!("  {}▌", app.nickname_buf), highlight_style())),
        Line::from(""),
        Line::from("Enter to confirm · Esc cancel"),
    ];
    frame.render_widget(Paragraph::new(lines).block(Block::default().title(" Nickname ").borders(Borders::ALL)), area);
}

fn draw_gyms(app: &App, frame: &mut Frame, area: Rect) {
    let gyms = default_gyms();
    let items: Vec<ListItem> = gyms.iter().enumerate().map(|(i, g)| {
        let selected = i == app.menu_index;
        let prefix = if selected { "> " } else { "  " };
        let earned = (app.save.badges & (1 << i)) != 0;
        let mark = if earned { "✓" } else { " " };
        let style = if selected { highlight_style() } else if earned { Style::default().fg(Color::Green) } else { Style::default() };
        ListItem::new(Line::from(Span::styled(format!("{prefix}[{mark}] {} — {} (Lv{}) ${}", g.name, g.badge_name, g.level, g.reward_money), style)))
    }).collect();
    frame.render_widget(bordered_list(" Gyms — need prior badges ", Color::Red, items), area);
}

fn draw_achievements(app: &App, frame: &mut Frame, area: Rect) {
    let mut lines = vec![Line::from(Span::styled("Achievements", highlight_style())), Line::from("")];
    let unlocked = app.save.achievements.unlock_summary();
    if unlocked.is_empty() {
        lines.push(Line::from("  (none yet — win battles, catch Pokemon, beat gyms)"));
    } else {
        for a in unlocked { lines.push(Line::from(format!("  ★ {a}"))); }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(format!("  Win streak: {} (best {})", app.save.stats.run_streak, app.save.stats.best_streak)));
    lines.push(Line::from(format!("  Caught: {}  Seen: {}  Badges: {}", app.save.dex.caught_count(), app.save.dex.seen_count(), app.save.badge_count())));
    lines.push(Line::from(format!("  Crits: {}  Shinies: {}  Evos: {}  Trainers: {}  Elite wins: {}",
        app.save.stats.critical_hits, app.save.stats.shinies_found, app.save.stats.evolutions,
        app.save.stats.trainers_defeated, app.save.stats.elite_wins)));
    lines.push(Line::from(format!("  Play time: {}", app.save.play_time_display())));
    lines.push(Line::from(""));
    lines.push(Line::from("Press q/Enter to return."));
    frame.render_widget(Paragraph::new(lines).block(Block::default().title(" Achievements ").borders(Borders::ALL)), area);
}

fn draw_box(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = if app.save.box_storage.is_empty() {
        vec![ListItem::new("  (PC box empty — deposit from Party with 'd')")]
    } else {
        app.save.box_storage.iter().enumerate().map(|(i, p)| {
            let selected = i == app.menu_index;
            let prefix = if selected { "> " } else { "  " };
            let style = if selected { highlight_style() } else { Style::default() };
            ListItem::new(Line::from(Span::styled(format!("{prefix}{}. {} Lv{}", i+1, p.display_name(), p.level), style)))
        }).collect()
    };
    frame.render_widget(bordered_list(" PC Box (Enter/w withdraw) ", Color::Cyan, items), area);
}

fn draw_type_chart(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled("Quick type reference", highlight_style())),
        Line::from("  Water > Fire > Grass > Water"),
        Line::from("  Electric > Water/Flying ; immune Ground"),
        Line::from("  Ground > Electric/Fire/Rock/Steel/Poison"),
        Line::from("  Fighting > Normal/Rock/Steel/Ice/Dark"),
        Line::from("  Ghost > Psychic/Ghost ; immune Normal"),
        Line::from("  Dragon > Dragon ; immune Fairy"),
        Line::from("  Fairy > Dragon/Fighting/Dark"),
        Line::from(""),
        Line::from("In battle, moves show ★ super / ▽ resist / ✕ immune when targeting foe."),
        Line::from(""),
        Line::from("Press q/Enter to return."),
    ];
    frame.render_widget(Paragraph::new(lines).block(Block::default().title(" Type Chart ").borders(Borders::ALL)), area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(Span::styled("Controls & features (v2.2)", highlight_style())),
        Line::from("  Adventure: wild/fish/trainer/rival, routes, gyms, Elite Four, mart, center"),
        Line::from("  Species dex: search, f=filter, b=battle, r=recruit, type ID then g=jump"),
        Line::from("  Battle: f fight, b ball, i item, p switch, r run, s sprites"),
        Line::from("  Party: d=PC box, n=nickname, t=move tutor"),
        Line::from("  Routes: Enter=grass, w=water, t=trainer · Weather affects damage"),
        Line::from("  Color sprites: PokeAPI PNG → half-block .cga (1025 species)"),
        Line::from("  Audio: original resources/track.mp3 loops (classic BGM)"),
        Line::from("  Auto-save every ~60s · Settings: volume, export save"),
        Line::from("  Shinies (~1/512), natures, IVs, evolution on level-up"),
        Line::from(""),
        Line::from("Press Enter/q to return."),
    ];
    frame.render_widget(Paragraph::new(text).block(Block::default().title(" Help ").borders(Borders::ALL)).wrap(Wrap { trim: false }), area);
}
