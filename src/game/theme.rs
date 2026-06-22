//! Shared UI theme — panels, colors, text helpers for a cleaner look.

use macroquad::prelude::*;

pub const C_PANEL: Color = Color::new(0.10, 0.13, 0.22, 0.94);
pub const C_PANEL_EDGE: Color = Color::new(0.55, 0.72, 0.95, 0.9);
pub const C_PANEL_EDGE_GOLD: Color = Color::new(0.95, 0.78, 0.28, 0.95);
pub const C_ACCENT: Color = Color::new(0.98, 0.82, 0.22, 1.0);
pub const C_TEXT: Color = Color::new(0.96, 0.97, 1.0, 1.0);
pub const C_TEXT_DIM: Color = Color::new(0.62, 0.68, 0.78, 1.0);
pub const C_SELECT: Color = Color::new(0.22, 0.38, 0.72, 0.85);
pub const C_HP_BG: Color = Color::new(0.12, 0.12, 0.14, 0.95);

pub fn draw_vignette() {
    let w = screen_width();
    let h = screen_height();
    // Corner darkening via edge strips
    for i in 0..8 {
        let a = 0.04 + i as f32 * 0.02;
        let c = Color::new(0.0, 0.0, 0.0, a);
        let t = i as f32 * 6.0;
        draw_rectangle(0.0, 0.0, w, t, c);
        draw_rectangle(0.0, h - t, w, t, c);
        draw_rectangle(0.0, 0.0, t, h, c);
        draw_rectangle(w - t, 0.0, t, h, c);
    }
}

/// Full-screen vertical gradient (menus / title / starter).
pub fn draw_gradient_bg(top: Color, bottom: Color, bands: i32) {
    let w = screen_width();
    let h = screen_height();
    let n = bands.max(8) as f32;
    for i in 0..bands.max(8) {
        let k = i as f32 / n;
        let c = Color::new(
            top.r + (bottom.r - top.r) * k,
            top.g + (bottom.g - top.g) * k,
            top.b + (bottom.b - top.b) * k,
            1.0,
        );
        draw_rectangle(0.0, i as f32 * (h / n), w, h / n + 1.0, c);
    }
}

/// Dim everything behind a modal dialog.
pub fn draw_modal_dim(alpha: f32) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.02, alpha.clamp(0.2, 0.85)),
    );
}

/// Rounded-ish panel via layered rects (macroquad has no real rounded rect).
pub fn draw_panel(x: f32, y: f32, w: f32, h: f32, gold_edge: bool) {
    // Drop shadow (slightly softer)
    draw_rectangle(x + 5.0, y + 6.0, w, h, Color::new(0.0, 0.0, 0.0, 0.35));
    // Body
    draw_rectangle(x, y, w, h, C_PANEL);
    // Left accent stripe
    let stripe = if gold_edge {
        Color::new(C_ACCENT.r, C_ACCENT.g, C_ACCENT.b, 0.55)
    } else {
        Color::new(C_PANEL_EDGE.r, C_PANEL_EDGE.g, C_PANEL_EDGE.b, 0.45)
    };
    draw_rectangle(x, y, 4.0, h, stripe);
    // Inner highlight top
    draw_rectangle(
        x + 5.0,
        y + 2.0,
        w - 7.0,
        3.0,
        Color::new(1.0, 1.0, 1.0, 0.09),
    );
    // Border
    let edge = if gold_edge {
        C_PANEL_EDGE_GOLD
    } else {
        C_PANEL_EDGE
    };
    draw_rectangle_lines(x, y, w, h, 2.5, edge);
    draw_rectangle_lines(
        x + 4.0,
        y + 3.0,
        w - 7.0,
        h - 6.0,
        1.0,
        Color::new(1.0, 1.0, 1.0, 0.10),
    );
}

pub fn draw_select_row(x: f32, y: f32, w: f32, h: f32, selected: bool) {
    if selected {
        let pulse = ((get_time() as f32 * 3.2).sin() * 0.5 + 0.5) * 0.12;
        draw_rectangle(
            x,
            y,
            w,
            h,
            Color::new(C_SELECT.r, C_SELECT.g, C_SELECT.b, 0.78 + pulse),
        );
        draw_rectangle(x, y, 4.0, h, C_ACCENT);
        // Right chevron cue
        let cy = y + h * 0.5;
        draw_triangle(
            vec2(x + w - 18.0, cy - 6.0),
            vec2(x + w - 8.0, cy),
            vec2(x + w - 18.0, cy + 6.0),
            Color::new(C_ACCENT.r, C_ACCENT.g, C_ACCENT.b, 0.85),
        );
        draw_rectangle_lines(x, y, w, h, 1.2, Color::new(1.0, 1.0, 1.0, 0.28 + pulse));
    }
}

pub fn draw_title_text(text: &str, x: f32, y: f32, size: f32) {
    // Soft shadow for readability
    draw_text(
        text,
        x + 2.0,
        y + 2.0,
        size,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    draw_text(text, x, y, size, C_ACCENT);
}

pub fn draw_body_text(text: &str, x: f32, y: f32, size: f32, dim: bool) {
    draw_text(text, x, y, size, if dim { C_TEXT_DIM } else { C_TEXT });
}

pub fn hp_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        Color::from_rgba(72, 210, 96, 255)
    } else if ratio > 0.25 {
        Color::from_rgba(240, 196, 48, 255)
    } else {
        Color::from_rgba(232, 64, 64, 255)
    }
}

/// Classic-style HP meter with label, level, numbers, and optional XP bar.
pub fn draw_status_box(
    x: f32,
    y: f32,
    w: f32,
    name: &str,
    level: u8,
    hp: i64,
    max_hp: i64,
    types: &[crate::pokemon::ElementType],
    xp_ratio: Option<f32>,
) {
    let h = if xp_ratio.is_some() { 86.0 } else { 72.0 };
    draw_panel(x, y, w, h, false);

    draw_body_text(&format!("{name}"), x + 14.0, y + 24.0, 20.0, false);
    draw_body_text(&format!("Lv{level}"), x + w - 52.0, y + 24.0, 18.0, true);

    // Type chips
    let mut tx = x + 14.0;
    for t in types.iter().take(2) {
        let (r, g, b) = t.rgb();
        let tw = measure_text(t.display_name(), None, 12, 1.0).width + 10.0;
        draw_rectangle(tx, y + 30.0, tw, 14.0, Color::from_rgba(r, g, b, 220));
        draw_text(t.display_name(), tx + 5.0, y + 41.0, 12.0, WHITE);
        tx += tw + 4.0;
    }

    let ratio = if max_hp > 0 {
        (hp.max(0) as f32 / max_hp as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let bar_x = x + 14.0;
    let bar_y = y + 50.0;
    let bar_w = w - 28.0;
    // Low HP pulse on the meter frame
    let danger = if ratio > 0.0 && ratio <= 0.25 {
        let p = ((get_time() as f32 * 5.0).sin() * 0.5 + 0.5) * 0.35;
        Color::new(0.9, 0.15, 0.15, p)
    } else {
        Color::new(0.0, 0.0, 0.0, 0.0)
    };
    draw_rectangle(bar_x - 1.0, bar_y - 1.0, bar_w + 2.0, 14.0, danger);
    draw_rectangle(bar_x, bar_y, bar_w, 12.0, C_HP_BG);
    let fill = (bar_w - 2.0) * ratio;
    let col = hp_color(ratio);
    draw_rectangle(bar_x + 1.0, bar_y + 1.0, fill, 10.0, col);
    draw_rectangle(
        bar_x + 1.0,
        bar_y + 1.0,
        fill,
        4.0,
        Color::new(1.0, 1.0, 1.0, 0.22),
    );
    draw_rectangle_lines(
        bar_x,
        bar_y,
        bar_w,
        12.0,
        1.0,
        Color::new(1.0, 1.0, 1.0, 0.35),
    );

    draw_body_text(
        &format!("{}/{}", hp.max(0), max_hp),
        x + w - 70.0,
        y + 68.0,
        13.0,
        true,
    );

    if let Some(xr) = xp_ratio {
        let xby = y + 72.0;
        draw_rectangle(bar_x, xby, bar_w, 6.0, Color::from_rgba(30, 30, 48, 255));
        draw_rectangle(
            bar_x,
            xby,
            (bar_w * xr.clamp(0.0, 1.0)).max(1.0),
            6.0,
            Color::from_rgba(72, 148, 255, 255),
        );
        draw_text("EXP", bar_x, xby + 14.0, 10.0, C_TEXT_DIM);
    }
}
