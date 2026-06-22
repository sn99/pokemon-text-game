//! Colored half-block sprites from PokeAPI PNGs (`.cga`: `char|rrggbb` tokens).

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use ratatui::style::Color;
use ratatui::text::{Line, Span};

use crate::pokemon::types::ElementType;

#[derive(Debug, Clone)]
pub struct ColorCell {
    pub ch: char,
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct ColorSprite {
    pub rows: Vec<Vec<ColorCell>>,
}

impl ColorSprite {
    pub fn height(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn to_lines(&self) -> Vec<Line<'static>> {
        self.rows
            .iter()
            .map(|row| {
                let spans: Vec<Span<'static>> = row
                    .iter()
                    .map(|c| {
                        Span::styled(
                            c.ch.to_string(),
                            ratatui::style::Style::default().fg(c.color),
                        )
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }

    pub fn flip_horizontal(&self) -> ColorSprite {
        let rows = self
            .rows
            .iter()
            .map(|row| row.iter().rev().cloned().collect())
            .collect();
        ColorSprite { rows }
    }

    /// Trim empty border rows/cols for tighter battle display.
    pub fn trim(&self) -> ColorSprite {
        if self.rows.is_empty() {
            return self.clone();
        }
        let mut rows = self.rows.clone();
        while rows.first().map(|r| r.iter().all(|c| c.ch == ' ')).unwrap_or(false) {
            rows.remove(0);
        }
        while rows.last().map(|r| r.iter().all(|c| c.ch == ' ')).unwrap_or(false) {
            rows.pop();
        }
        let max_w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut left = max_w;
        let mut right = 0usize;
        for r in &rows {
            for (i, c) in r.iter().enumerate() {
                if c.ch != ' ' {
                    left = left.min(i);
                    right = right.max(i + 1);
                }
            }
        }
        if left < right {
            for r in &mut rows {
                if r.len() > left {
                    *r = r[left..r.len().min(right)].to_vec();
                }
            }
        }
        // Cap height for terminal
        if rows.len() > 14 {
            rows.truncate(14);
        }
        ColorSprite { rows }
    }

    /// Side-by-side battle frame as colored lines.
    pub fn battle_lines(
        player: &ColorSprite,
        foe: &ColorSprite,
        p_name: &str,
        f_name: &str,
        shiny_p: bool,
        shiny_f: bool,
    ) -> Vec<Line<'static>> {
        let gap = "  VS  ";
        let mut out = Vec::new();
        let p_label = if shiny_p {
            format!("✦{p_name}")
        } else {
            format!("[{p_name}]")
        };
        let f_label = if shiny_f {
            format!("✦{f_name}")
        } else {
            format!("[{f_name}]")
        };
        out.push(Line::from(Span::styled(
            format!("{p_label}{gap}{f_label}"),
            ratatui::style::Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )));
        let h = player.height().max(foe.height());
        for i in 0..h {
            let mut spans = Vec::new();
            if let Some(row) = player.rows.get(i) {
                for c in row {
                    let col = if shiny_p {
                        boost_shiny(c.color)
                    } else {
                        c.color
                    };
                    spans.push(Span::styled(
                        c.ch.to_string(),
                        ratatui::style::Style::default().fg(col),
                    ));
                }
            }
            spans.push(Span::styled(
                gap,
                ratatui::style::Style::default().fg(Color::DarkGray),
            ));
            if let Some(row) = foe.rows.get(i) {
                for c in row {
                    let col = if shiny_f {
                        boost_shiny(c.color)
                    } else {
                        c.color
                    };
                    spans.push(Span::styled(
                        c.ch.to_string(),
                        ratatui::style::Style::default().fg(col),
                    ));
                }
            }
            out.push(Line::from(spans));
        }
        out
    }
}

fn boost_shiny(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(40).min(255),
            g.saturating_add(20).min(255),
            b.saturating_add(60).min(255),
        ),
        other => other,
    }
}

/// Fallback colored block art when no `.cga` exists.
pub fn procedural_color_sprite(id: u16, primary: ElementType) -> ColorSprite {
    let col = type_rgb(primary);
    let seed = id as usize;
    let mut rows = Vec::new();
    let h = 10usize;
    let w = 18usize;
    for y in 0..h {
        let mut row = Vec::new();
        for x in 0..w {
            let dx = x as i32 - 9;
            let dy = y as i32 - 5;
            let r2 = dx * dx + dy * dy;
            let ch = if r2 < 8 + (seed % 5) as i32 {
                if (x + y + seed) % 3 == 0 {
                    '█'
                } else if y < 4 {
                    '▄'
                } else {
                    '▀'
                }
            } else if r2 < 14 + (seed % 3) as i32 {
                '░'
            } else {
                ' '
            };
            let shade = if ch == ' ' {
                Color::Reset
            } else {
                let factor = 1.0 - (r2 as f32 / 40.0).clamp(0.0, 0.5);
                match col {
                    Color::Rgb(r, g, b) => Color::Rgb(
                        ((r as f32) * factor) as u8,
                        ((g as f32) * factor) as u8,
                        ((b as f32) * factor) as u8,
                    ),
                    c => c,
                }
            };
            row.push(ColorCell { ch, color: shade });
        }
        rows.push(row);
    }
    ColorSprite { rows }.trim()
}

fn type_rgb(t: ElementType) -> Color {
    use ElementType::*;
    match t {
        Normal => Color::Rgb(168, 168, 120),
        Fire => Color::Rgb(240, 128, 48),
        Water => Color::Rgb(104, 144, 240),
        Electric => Color::Rgb(248, 208, 48),
        Grass => Color::Rgb(120, 200, 80),
        Ice => Color::Rgb(152, 216, 216),
        Fighting => Color::Rgb(192, 48, 40),
        Poison => Color::Rgb(160, 64, 160),
        Ground => Color::Rgb(224, 192, 104),
        Flying => Color::Rgb(168, 144, 240),
        Psychic => Color::Rgb(248, 88, 136),
        Bug => Color::Rgb(168, 184, 32),
        Rock => Color::Rgb(184, 160, 56),
        Ghost => Color::Rgb(112, 88, 152),
        Dragon => Color::Rgb(112, 56, 248),
        Dark => Color::Rgb(112, 88, 72),
        Steel => Color::Rgb(184, 184, 208),
        Fairy => Color::Rgb(238, 153, 172),
    }
}

static CACHE: OnceLock<HashMap<u16, ColorSprite>> = OnceLock::new();

fn load_all() -> HashMap<u16, ColorSprite> {
    let mut map = HashMap::new();
    let dir = Path::new("resources/sprites/color");
    let Ok(entries) = fs::read_dir(dir) else {
        return map;
    };
    for ent in entries.flatten() {
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("cga") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let Ok(id) = stem.parse::<u16>() else {
            continue;
        };
        if let Ok(text) = fs::read_to_string(&path) {
            if let Some(sp) = parse_cga(&text) {
                map.insert(id, sp.trim());
            }
        }
    }
    map
}

fn parse_cga(text: &str) -> Option<ColorSprite> {
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            rows.push(Vec::new());
            continue;
        }
        let mut row = Vec::new();
        // Tokens may be `█|ffcc00` or split oddly; scan with regex-like split
        for tok in line.split_whitespace() {
            let Some((ch_s, hex)) = tok.split_once('|') else {
                continue;
            };
            let ch = ch_s.chars().next().unwrap_or(' ');
            let hex = if hex.len() >= 6 { &hex[..6] } else { continue };
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(200);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(200);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(200);
            if ch == ' ' && r == 0 && g == 0 && b == 0 {
                row.push(ColorCell {
                    ch: ' ',
                    color: Color::Reset,
                });
                continue;
            }
            row.push(ColorCell {
                ch,
                color: Color::Rgb(r, g, b),
            });
        }
        while row.last().map(|c| c.ch == ' ').unwrap_or(false) {
            row.pop();
        }
        rows.push(row);
    }
    while rows.first().map(|r| r.is_empty()).unwrap_or(false) {
        rows.remove(0);
    }
    while rows.last().map(|r| r.is_empty()).unwrap_or(false) {
        rows.pop();
    }
    if rows.is_empty() {
        None
    } else {
        Some(ColorSprite { rows })
    }
}

pub fn load_color_sprite(id: u16) -> Option<ColorSprite> {
    CACHE.get_or_init(load_all).get(&id).cloned()
}

/// Prefer file sprite; fall back to procedural colored art.
pub fn color_sprite_for_species(id: u16, primary: ElementType) -> ColorSprite {
    load_color_sprite(id).unwrap_or_else(|| procedural_color_sprite(id, primary))
}

pub fn color_sprite_count() -> usize {
    CACHE.get_or_init(load_all).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal() {
        let s = parse_cga("█|ffcc00 ▄|00aaff\n▀|ff0000\n").unwrap();
        assert_eq!(s.height(), 2);
    }

    #[test]
    fn procedural_non_empty() {
        let s = procedural_color_sprite(25, ElementType::Electric);
        assert!(!s.is_empty());
    }
}
