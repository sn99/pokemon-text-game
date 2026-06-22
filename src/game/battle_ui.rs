//! Battle screen rendering — polished panels, HP boxes, menus.

use macroquad::prelude::*;

use crate::pokemon::PokemonInstance;

use super::assets::SpriteCache;
use super::theme::{
    draw_modal_dim, draw_panel, draw_select_row, draw_title_text, C_ACCENT, C_TEXT, C_TEXT_DIM,
};

pub fn draw_battle_backdrop() {
    let w = screen_width();
    let h = screen_height();
    let t = get_time() as f32;

    // Sky gradient
    for i in 0..28 {
        let k = i as f32 / 28.0;
        let c = Color::from_rgba(
            (48.0 + k * 70.0) as u8,
            (92.0 + k * 90.0) as u8,
            (150.0 + k * 55.0) as u8,
            255,
        );
        draw_rectangle(
            0.0,
            i as f32 * (h * 0.52 / 28.0),
            w,
            h * 0.52 / 28.0 + 1.0,
            c,
        );
    }

    // Simple cloud blobs (cheap parallax)
    let cx = ((t * 12.0) % (w + 120.0)) - 60.0;
    draw_ellipse(
        cx,
        h * 0.12,
        70.0,
        22.0,
        0.0,
        Color::from_rgba(255, 255, 255, 55),
    );
    draw_ellipse(
        cx + 40.0,
        h * 0.11,
        50.0,
        18.0,
        0.0,
        Color::from_rgba(255, 255, 255, 45),
    );
    draw_ellipse(
        (cx * 0.6 + w * 0.4) % (w + 100.0),
        h * 0.20,
        90.0,
        26.0,
        0.0,
        Color::from_rgba(255, 255, 255, 40),
    );
    // Second slower cloud layer
    let cx2 = ((t * 7.0 + 200.0) % (w + 140.0)) - 70.0;
    draw_ellipse(
        cx2,
        h * 0.16,
        60.0,
        18.0,
        0.0,
        Color::from_rgba(255, 255, 255, 35),
    );

    // Distant hills
    draw_ellipse(
        w * 0.2,
        h * 0.48,
        280.0,
        70.0,
        0.0,
        Color::from_rgba(58, 120, 72, 255),
    );
    draw_ellipse(
        w * 0.65,
        h * 0.50,
        340.0,
        80.0,
        0.0,
        Color::from_rgba(48, 108, 62, 255),
    );
    draw_ellipse(
        w * 0.45,
        h * 0.51,
        200.0,
        50.0,
        0.0,
        Color::from_rgba(42, 96, 54, 255),
    );

    // Ground bands with subtle stripes
    draw_rectangle(
        0.0,
        h * 0.52,
        w,
        h * 0.48,
        Color::from_rgba(78, 148, 68, 255),
    );
    draw_rectangle(
        0.0,
        h * 0.62,
        w,
        h * 0.38,
        Color::from_rgba(68, 132, 58, 255),
    );
    for i in 0..6 {
        let gx = (i as f32 * 140.0 + (t * 8.0).sin() * 4.0) % (w + 40.0) - 20.0;
        draw_rectangle(
            gx,
            h * 0.54,
            30.0,
            h * 0.46,
            Color::from_rgba(72, 140, 62, 40),
        );
    }

    // Battle platforms (ellipses)
    let bob = (t * 1.2).sin() * 1.5;
    draw_ellipse(
        w * 0.70,
        h * 0.42 + bob,
        130.0,
        28.0,
        0.0,
        Color::from_rgba(52, 108, 48, 230),
    );
    draw_ellipse(
        w * 0.70,
        h * 0.42 + bob,
        110.0,
        20.0,
        0.0,
        Color::from_rgba(90, 160, 70, 120),
    );
    draw_ellipse(
        w * 0.28,
        h * 0.68,
        150.0,
        32.0,
        0.0,
        Color::from_rgba(52, 108, 48, 230),
    );
    draw_ellipse(
        w * 0.28,
        h * 0.68,
        128.0,
        22.0,
        0.0,
        Color::from_rgba(90, 160, 70, 120),
    );

    // Soft vignette corners
    for i in 0..6 {
        let a = 0.05 + i as f32 * 0.03;
        draw_rectangle(
            0.0,
            0.0,
            w,
            8.0 + i as f32 * 5.0,
            Color::new(0.0, 0.0, 0.0, a),
        );
    }
}

pub fn draw_pokemon_sprite(
    sprites: &SpriteCache,
    species_id: u16,
    x: f32,
    y: f32,
    size: f32,
    back: bool,
    shake: f32,
) {
    let tex = if back {
        sprites.back(species_id)
    } else {
        sprites.front(species_id)
    };
    let t = get_time() as f32;
    let idle = (t * 2.2 + species_id as f32 * 0.1).sin() * 3.0;
    let sx = x + shake;
    let sy = y + idle;

    // Ground shadow under sprite
    draw_ellipse(
        sx + size * 0.5,
        sy + size * 0.92,
        size * 0.38,
        size * 0.10,
        0.0,
        Color::from_rgba(0, 0, 0, 55),
    );

    draw_texture_ex(
        tex,
        sx,
        sy,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        },
    );
}

pub(crate) fn truncate_chars(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let take = max_chars.saturating_sub(1);
    let mut out: String = s.chars().take(take).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::truncate_chars;

    #[test]
    fn truncate_chars_respects_utf8() {
        let s = "あいうえおかきくけこ";
        let t = truncate_chars(s, 5);
        assert!(t.ends_with('…'));
        assert_eq!(t.chars().count(), 5); // 4 chars + ellipsis
    }

    #[test]
    fn truncate_chars_short_unchanged() {
        assert_eq!(truncate_chars("hi", 10), "hi");
    }
}

pub fn draw_battle_log(lines: &[String]) {
    let w = screen_width();
    let h = screen_height();
    let box_h = 112.0;
    let x = 14.0;
    let y = h - box_h - 12.0;
    let bw = w - 310.0;
    draw_panel(x, y, bw.max(200.0), box_h, false);
    draw_title_text("Battle", x + 14.0, y + 22.0, 16.0);

    let start = lines.len().saturating_sub(4);
    let slice = &lines[start..];
    for (i, line) in slice.iter().enumerate() {
        let ly = y + 42.0 + i as f32 * 16.0;
        let is_latest = i + 1 == slice.len();
        // Truncate visually if very long
        let shown = truncate_chars(line, 72);
        let col = if is_latest {
            WHITE
        } else {
            Color::new(C_TEXT.r, C_TEXT.g, C_TEXT.b, 0.72)
        };
        if is_latest {
            draw_rectangle(
                x + 8.0,
                ly - 12.0,
                bw.max(200.0) - 16.0,
                16.0,
                Color::new(1.0, 1.0, 1.0, 0.06),
            );
        }
        draw_text(&shown, x + 16.0, ly, 16.0, col);
    }
}

pub fn draw_battle_menu(
    menu_idx: usize,
    in_moves: bool,
    move_idx: usize,
    player: &PokemonInstance,
    can_catch: bool,
    foe_types: &[crate::pokemon::ElementType],
) {
    let w = screen_width();
    let h = screen_height();
    let mw = 280.0;
    let mh = 228.0;
    let mx = w - mw - 16.0;
    let my = h - mh - 12.0;
    draw_panel(mx, my, mw, mh, true);

    if in_moves {
        draw_title_text("Moves", mx + 16.0, my + 26.0, 20.0);
        for (i, mv) in player.moves.iter().enumerate() {
            let ry = my + 42.0 + i as f32 * 38.0;
            draw_select_row(mx + 10.0, ry - 14.0, mw - 20.0, 32.0, i == move_idx);

            let mult = crate::pokemon::type_effectiveness(mv.data.move_type, foe_types);
            let hint = if mult >= 2.0 {
                "SE"
            } else if mult == 0.0 {
                "—"
            } else if mult <= 0.5 {
                "NVE"
            } else {
                ""
            };
            let (r, g, b) = mv.data.move_type.rgb();
            // Type badge
            draw_rectangle(
                mx + 16.0,
                ry - 8.0,
                52.0,
                16.0,
                Color::from_rgba(r, g, b, 230),
            );
            draw_text(
                &mv.data.move_type.display_name()[..mv.data.move_type.display_name().len().min(6)],
                mx + 20.0,
                ry + 4.0,
                12.0,
                WHITE,
            );

            let pwr = if mv.data.power > 0 {
                format!("Pwr{}", mv.data.power)
            } else {
                "Status".into()
            };
            let label = format!(
                "{}  {}/{}  {}",
                mv.data.name, mv.current_pp, mv.data.pp, pwr
            );
            draw_text(&label, mx + 74.0, ry + 2.0, 14.0, C_TEXT);
            if !hint.is_empty() {
                let hc = if mult >= 2.0 {
                    Color::from_rgba(100, 255, 120, 255)
                } else if mult == 0.0 {
                    Color::from_rgba(255, 100, 100, 255)
                } else {
                    Color::from_rgba(255, 200, 80, 255)
                };
                draw_text(hint, mx + mw - 48.0, ry + 2.0, 13.0, hc);
            }
        }
        draw_text(
            "[Esc] Back · 1-4 quick move",
            mx + 16.0,
            my + mh - 14.0,
            12.0,
            C_TEXT_DIM,
        );
    } else {
        draw_title_text("Command", mx + 16.0, my + 26.0, 20.0);
        let opts = [
            ("Fight", "Choose a move", "1"),
            (
                "Bag",
                if can_catch { "Ball / potion" } else { "Items" },
                "2",
            ),
            ("Switch", "Change lead", "3"),
            ("Run", "Try to flee", "4"),
        ];
        for (i, (o, sub, key)) in opts.iter().enumerate() {
            let ry = my + 48.0 + i as f32 * 36.0;
            draw_select_row(mx + 10.0, ry - 14.0, mw - 20.0, 34.0, i == menu_idx);
            draw_text(key, mx + 16.0, ry + 10.0, 11.0, C_ACCENT);
            draw_text(
                o,
                mx + 32.0,
                ry + 2.0,
                20.0,
                if i == menu_idx { WHITE } else { C_TEXT },
            );
            draw_text(sub, mx + 32.0, ry + 18.0, 12.0, C_TEXT_DIM);
        }
        draw_text(
            "↑↓ / 1-4 · Enter confirm",
            mx + 16.0,
            my + mh - 14.0,
            12.0,
            C_TEXT_DIM,
        );
    }
}

pub fn draw_switch_menu(
    switch_idx: usize,
    party: &[crate::pokemon::PokemonInstance],
    active_idx: usize,
) {
    let w = screen_width();
    let h = screen_height();
    let mw = 340.0;
    let mh = 60.0 + party.len().max(1) as f32 * 42.0;
    let mx = w / 2.0 - mw / 2.0;
    let my = h / 2.0 - mh / 2.0;
    draw_modal_dim(0.48);
    draw_panel(mx, my, mw, mh.min(320.0), true);
    draw_title_text("Switch", mx + 110.0, my + 30.0, 24.0);
    for (i, p) in party.iter().enumerate() {
        let ry = my + 50.0 + i as f32 * 40.0;
        draw_select_row(mx + 10.0, ry - 14.0, mw - 20.0, 36.0, i == switch_idx);
        let mark = if i == active_idx { "★ " } else { "  " };
        let fnt = if p.is_fainted() { " [FNT]" } else { "" };
        draw_text(
            &format!(
                "{}{} Lv{}{}  {}/{}",
                mark,
                p.display_name(),
                p.level,
                fnt,
                p.current_hp.max(0),
                p.max_hp
            ),
            mx + 24.0,
            ry + 6.0,
            16.0,
            if p.is_fainted() { GRAY } else { C_TEXT },
        );
    }
}

pub fn draw_bag_menu(bag_idx: usize, balls: u32, potions: u32, catch_hint: Option<&str>) {
    let w = screen_width();
    let h = screen_height();
    let mw = 340.0;
    let mh = 220.0;
    let mx = w / 2.0 - mw / 2.0;
    let my = h / 2.0 - mh / 2.0;
    draw_modal_dim(0.52);
    draw_panel(mx, my, mw, mh, true);
    draw_title_text("Bag", mx + 20.0, my + 32.0, 26.0);

    let ball_sub = catch_hint.unwrap_or("Catch wild Pokémon");
    let items = [
        (format!("Poke Ball    ×{balls}"), ball_sub),
        (format!("Potion       ×{potions}"), "Restore 40 HP"),
        ("Cancel".into(), "Close bag"),
    ];
    for (i, (it, sub)) in items.iter().enumerate() {
        let ry = my + 58.0 + i as f32 * 40.0;
        draw_select_row(mx + 12.0, ry - 14.0, mw - 24.0, 36.0, i == bag_idx);
        draw_text(it, mx + 28.0, ry + 2.0, 18.0, C_TEXT);
        draw_text(sub, mx + 28.0, ry + 18.0, 13.0, C_TEXT_DIM);
    }
    draw_text(
        "Tip: weaken the foe before throwing balls",
        mx + 20.0,
        my + mh - 22.0,
        12.0,
        C_TEXT_DIM,
    );
}

pub fn draw_overlay_banner(text: &str, color: Color) {
    let w = screen_width();
    let h = screen_height();
    let pulse = ((get_time() as f32 * 4.0).sin() * 0.5 + 0.5) * 0.08;
    draw_modal_dim(0.35 + pulse * 0.5);
    let tw = measure_text(text, None, 42, 1.0).width + 72.0;
    let x = (w - tw) / 2.0;
    let y = h * 0.36;
    draw_panel(x, y, tw, 72.0, true);
    draw_rectangle(x + 6.0, y + 6.0, tw - 12.0, 4.0, C_ACCENT);
    draw_text(text, x + 36.0, y + 50.0, 42.0, color);
}

/// Floating damage / heal numbers that rise and fade.
pub struct FloatText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub life: f32,
    pub max_life: f32,
}

impl FloatText {
    pub fn damage(x: f32, y: f32, amount: i64) -> Self {
        Self {
            x,
            y,
            text: format!("-{amount}"),
            color: Color::from_rgba(255, 90, 90, 255),
            life: 1.0,
            max_life: 1.0,
        }
    }

    pub fn heal(x: f32, y: f32, amount: i64) -> Self {
        Self {
            x,
            y,
            text: format!("+{amount}"),
            color: Color::from_rgba(90, 230, 120, 255),
            life: 1.0,
            max_life: 1.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.life -= dt;
        self.y -= 28.0 * dt;
    }

    pub fn alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn draw(&self) {
        let a = (self.life / self.max_life).clamp(0.0, 1.0);
        let c = Color::new(self.color.r, self.color.g, self.color.b, a);
        draw_text(
            &self.text,
            self.x + 1.0,
            self.y + 1.0,
            26.0,
            Color::new(0.0, 0.0, 0.0, a * 0.6),
        );
        draw_text(&self.text, self.x, self.y, 26.0, c);
    }
}

pub fn draw_float_texts(floats: &[FloatText]) {
    for f in floats {
        f.draw();
    }
}
