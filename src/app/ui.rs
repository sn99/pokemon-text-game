use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use super::{App, FormField, FormKind, Screen};
use pokemon_text_game::Pokemon;

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
                highlight_style()
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

fn bordered_list<'a>(title: &'a str, border: Color, items: Vec<ListItem<'a>>) -> List<'a> {
    List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border)),
    )
}

fn pokemon_list_items(pokemons: &[Pokemon], detailed: bool) -> Vec<ListItem<'_>> {
    pokemons
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let line = if detailed {
                format!(
                    "{}. {}  HP:{}  type:{}  moves:{}",
                    i + 1,
                    p.name,
                    p.health,
                    p.pokemon_type,
                    p.moves_name.join(", ")
                )
            } else {
                format!(
                    "{}. {}  HP:{}  moves:{}",
                    i + 1,
                    p.name,
                    p.health,
                    p.moves_name.join(", ")
                )
            };
            ListItem::new(line)
        })
        .collect()
}

fn highlight_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

pub fn draw(app: &mut App, frame: &mut Frame) {
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

    match app.screen {
        Screen::MainMenu => draw_simple_menu(
            frame,
            chunks[1],
            " Main Menu ",
            Color::Green,
            &["Play game", "Enter Pokedex", "Exit"],
            app.menu_index,
        ),
        Screen::PlayMode => draw_simple_menu(
            frame,
            chunks[1],
            " Play Mode ",
            Color::Green,
            &["Against Computer", "Against Human", "Back"],
            app.menu_index,
        ),
        Screen::SelectPokemon { slot } => draw_pokemon_picker(app, frame, chunks[1], slot.title()),
        Screen::Battle | Screen::BattleOver => draw_battle(app, frame, chunks[1]),
        Screen::PokedexMenu => draw_simple_menu(
            frame,
            chunks[1],
            " Pokedex ",
            Color::LightBlue,
            &[
                "Create new Pokemon",
                "Edit existing Pokemon",
                "Delete a Pokemon",
                "Main menu",
            ],
            app.menu_index,
        ),
        Screen::PokedexList { action } => {
            draw_pokemon_picker_detailed(app, frame, chunks[1], action.title())
        }
        Screen::FormInput => draw_form(app, frame, chunks[1]),
        Screen::Message => draw_message_popup(app, frame, area),
    }

    let status_text = if !app.status.is_empty() {
        app.status.as_str()
    } else {
        "↑/↓ navigate · Enter select · q/Esc back · Ctrl+C quit"
    };
    frame.render_widget(help_line(status_text), chunks[2]);
}

fn draw_simple_menu(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    border: Color,
    labels: &[&str],
    selected: usize,
) {
    let list = bordered_list(title, border, menu_items(labels, selected));
    frame.render_widget(list, area);
}

fn draw_pokemon_picker(app: &mut App, frame: &mut Frame, area: Rect, title: &str) {
    let items = pokemon_list_items(&app.team.pokeball, false);
    let list = bordered_list(title, Color::Magenta, items)
        .highlight_style(highlight_style())
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_pokemon_picker_detailed(app: &mut App, frame: &mut Frame, area: Rect, title: &str) {
    let items = pokemon_list_items(&app.team.pokeball, true);
    let list = bordered_list(title, Color::Magenta, items)
        .highlight_style(highlight_style())
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_hp_gauge(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    pokemon: &Pokemon,
    max_hp: i64,
    color: Color,
) {
    let gauge = Gauge::default()
        .block(Block::default().title(format!(" {label}: {} ", pokemon.name)))
        .gauge_style(Style::default().fg(color))
        .ratio(pokemon.health_ratio(max_hp))
        .label(format!("{} / {}", pokemon.health.max(0), max_hp));
    frame.render_widget(gauge, area);
}

fn draw_battle(app: &App, frame: &mut Frame, area: Rect) {
    let Some(b) = &app.battle else {
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

    draw_hp_gauge(
        frame,
        rows[0],
        b.side_label(true),
        &b.player1,
        b.player1_max_hp,
        Color::Green,
    );
    draw_hp_gauge(
        frame,
        rows[1],
        b.side_label(false),
        &b.player2,
        b.player2_max_hp,
        Color::Red,
    );

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

    if app.screen == Screen::BattleOver {
        let win = b.winner_text.as_deref().unwrap_or("Battle over");
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(win, highlight_style())),
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
        return;
    }

    let whose = b.turn_prompt();
    let moves = b.attacker_moves();
    let items: Vec<ListItem> = moves
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let selected = i == b.move_cursor;
            let prefix = if selected { "> " } else { "  " };
            let style = if selected {
                highlight_style()
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{prefix}{}. {m}", i + 1),
                style,
            )))
        })
        .collect();
    let title = format!(" Moves — {whose} ");
    let list = bordered_list(&title, Color::Blue, items);
    frame.render_widget(list, rows[3]);
}

fn draw_form(app: &App, frame: &mut Frame, area: Rect) {
    let Some(form) = &app.form else {
        return;
    };
    let title = match form.kind {
        FormKind::CreatePokemon => " Create Pokemon ",
        FormKind::EditPokemon { .. } => " Edit Pokemon ",
    };

    let field_style = |f: FormField| {
        if form.field == f {
            highlight_style()
        } else {
            Style::default().fg(Color::Gray)
        }
    };
    let val = |f: FormField| {
        let s = form.field_value(f);
        let display = if s.is_empty() { " " } else { s };
        let cursor = if form.field == f { "▌" } else { "" };
        format!("{display}{cursor}")
    };

    let mut lines = Vec::new();
    for field in FormField::ALL {
        lines.push(Line::from(Span::styled(
            format!("  {}:", field.label()),
            field_style(field),
        )));
        lines.push(Line::from(format!("    {}", val(field))));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "Tab/↑↓ change field · Enter save · Esc cancel",
        Style::default().fg(Color::DarkGray),
    )));

    if let Some(err) = &form.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Error: {err}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn draw_message_popup(app: &App, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup);
    let para = Paragraph::new(vec![
        Line::from(""),
        Line::from(app.message.as_str()),
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
