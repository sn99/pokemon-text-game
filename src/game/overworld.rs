//! Overworld map drawing — richer tiles, HUD, player, NPCs, dialogue.

use macroquad::prelude::*;

use crate::world::{world_props, PropKind, Tile, WorldProp, MAP_H, MAP_W, TILE_PX};

use super::assets::SpriteCache;
use super::battle_ui::draw_pokemon_sprite;
use super::theme::{draw_modal_dim, draw_panel, draw_title_text, C_ACCENT, C_TEXT, C_TEXT_DIM};

pub fn tile_color(t: Tile) -> Color {
    match t {
        Tile::Grass => Color::from_rgba(98, 168, 78, 255),
        Tile::TallGrass => Color::from_rgba(38, 108, 48, 255),
        Tile::Path => Color::from_rgba(198, 176, 124, 255),
        Tile::Water => Color::from_rgba(48, 124, 208, 255),
        Tile::Tree => Color::from_rgba(36, 96, 44, 255),
        Tile::Building => Color::from_rgba(188, 86, 86, 255),
        Tile::Floor => Color::from_rgba(218, 208, 188, 255),
        Tile::Door => Color::from_rgba(108, 72, 44, 255),
        Tile::Counter => Color::from_rgba(150, 112, 74, 255),
        Tile::Flower => Color::from_rgba(108, 168, 86, 255),
        Tile::Mart => Color::from_rgba(64, 112, 192, 255),
        Tile::MartDoor => Color::from_rgba(82, 64, 44, 255),
        Tile::Sand => Color::from_rgba(220, 200, 140, 255),
        Tile::Fence => Color::from_rgba(120, 88, 48, 255),
        Tile::Sign => Color::from_rgba(188, 168, 118, 255),
    }
}

fn checker_tint(tx: usize, ty: usize, base: Color, amt: f32) -> Color {
    if (tx + ty) % 2 == 0 {
        Color::new(
            (base.r + amt).min(1.0),
            (base.g + amt).min(1.0),
            (base.b + amt).min(1.0),
            base.a,
        )
    } else {
        base
    }
}

pub fn draw_map(map: &[Vec<Tile>], cam_x: f32, cam_y: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let t = get_time() as f32;

    for ty in 0..MAP_H {
        for tx in 0..MAP_W {
            let tile = map[ty][tx];
            let x = tx as f32 * TILE_PX - cam_x;
            let y = ty as f32 * TILE_PX - cam_y;
            if x + TILE_PX < 0.0 || y + TILE_PX < 0.0 || x > sw || y > sh {
                continue;
            }

            let base = checker_tint(tx, ty, tile_color(tile), 0.03);
            draw_rectangle(x, y, TILE_PX, TILE_PX, base);

            match tile {
                Tile::Grass => {
                    for i in 0..2 {
                        let ox = x + 8.0 + i as f32 * 12.0 + ((tx + i) % 3) as f32;
                        draw_line(
                            ox,
                            y + TILE_PX - 2.0,
                            ox + 1.0,
                            y + 14.0,
                            1.2,
                            Color::from_rgba(70, 140, 55, 90),
                        );
                    }
                }
                Tile::TallGrass => {
                    for i in 0..4 {
                        let sway = (t * 2.5 + tx as f32 * 0.7 + i as f32).sin() * 2.0;
                        let ox = x + 5.0 + i as f32 * 7.0;
                        draw_line(
                            ox,
                            y + TILE_PX - 2.0,
                            ox + sway,
                            y + 6.0,
                            2.4,
                            Color::from_rgba(28, 88, 34, 230),
                        );
                        draw_line(
                            ox + 1.0,
                            y + TILE_PX - 2.0,
                            ox + sway + 1.0,
                            y + 10.0,
                            1.4,
                            Color::from_rgba(60, 150, 55, 180),
                        );
                    }
                }
                Tile::Water => {
                    let deep = Color::from_rgba(36, 96, 180, 255);
                    draw_rectangle(x, y + TILE_PX * 0.55, TILE_PX, TILE_PX * 0.45, deep);
                    for wi in 0..2 {
                        let wave = (t * 2.2 + tx as f32 * 0.55 + wi as f32 * 1.2).sin() * 2.5;
                        let wy = y + 10.0 + wi as f32 * 10.0 + wave;
                        draw_line(
                            x + 3.0,
                            wy,
                            x + TILE_PX - 3.0,
                            wy,
                            1.6,
                            Color::from_rgba(190, 230, 255, 110),
                        );
                    }
                    if (tx + ty) % 5 == 0 {
                        let gx = x + 8.0 + (t * 1.5).sin() * 4.0;
                        draw_circle(gx, y + 12.0, 2.0, Color::from_rgba(255, 255, 255, 90));
                    }
                }
                Tile::Tree => {
                    draw_rectangle(
                        x + 12.0,
                        y + 18.0,
                        8.0,
                        12.0,
                        Color::from_rgba(92, 62, 38, 255),
                    );
                    draw_circle(x + 16.0, y + 14.0, 13.0, Color::from_rgba(32, 96, 40, 255));
                    draw_circle(x + 10.0, y + 16.0, 9.0, Color::from_rgba(28, 82, 34, 255));
                    draw_circle(x + 22.0, y + 16.0, 9.0, Color::from_rgba(38, 108, 44, 255));
                    draw_circle(x + 16.0, y + 10.0, 7.0, Color::from_rgba(48, 128, 52, 200));
                }
                Tile::Building => {
                    draw_rectangle(
                        x + 1.0,
                        y + 8.0,
                        TILE_PX - 2.0,
                        TILE_PX - 8.0,
                        Color::from_rgba(220, 118, 118, 255),
                    );
                    draw_rectangle(
                        x + 6.0,
                        y + 14.0,
                        7.0,
                        7.0,
                        Color::from_rgba(180, 220, 255, 255),
                    );
                    draw_rectangle(
                        x + 19.0,
                        y + 14.0,
                        7.0,
                        7.0,
                        Color::from_rgba(180, 220, 255, 255),
                    );
                    draw_triangle(
                        vec2(x - 1.0, y + 10.0),
                        vec2(x + TILE_PX / 2.0, y - 2.0),
                        vec2(x + TILE_PX + 1.0, y + 10.0),
                        Color::from_rgba(168, 48, 48, 255),
                    );
                    draw_triangle(
                        vec2(x + 2.0, y + 10.0),
                        vec2(x + TILE_PX / 2.0, y + 1.0),
                        vec2(x + TILE_PX - 2.0, y + 10.0),
                        Color::from_rgba(200, 68, 68, 255),
                    );
                    draw_rectangle(x + 13.0, y + 22.0, 6.0, 6.0, WHITE);
                    draw_rectangle(
                        x + 14.5,
                        y + 20.0,
                        3.0,
                        10.0,
                        Color::from_rgba(220, 40, 40, 255),
                    );
                    draw_rectangle(
                        x + 11.5,
                        y + 23.0,
                        9.0,
                        3.0,
                        Color::from_rgba(220, 40, 40, 255),
                    );
                }
                Tile::Door => {
                    draw_rectangle(
                        x + 6.0,
                        y + 6.0,
                        TILE_PX - 12.0,
                        TILE_PX - 6.0,
                        Color::from_rgba(72, 48, 28, 255),
                    );
                    draw_rectangle(
                        x + 9.0,
                        y + 10.0,
                        TILE_PX - 18.0,
                        TILE_PX - 12.0,
                        Color::from_rgba(52, 34, 20, 255),
                    );
                    draw_circle(x + TILE_PX - 12.0, y + 20.0, 2.0, C_ACCENT);
                }
                Tile::Mart => {
                    draw_rectangle(
                        x + 1.0,
                        y + 8.0,
                        TILE_PX - 2.0,
                        TILE_PX - 8.0,
                        Color::from_rgba(88, 148, 220, 255),
                    );
                    draw_rectangle(
                        x + 6.0,
                        y + 14.0,
                        8.0,
                        8.0,
                        Color::from_rgba(200, 230, 255, 255),
                    );
                    draw_rectangle(
                        x + 22.0,
                        y + 14.0,
                        8.0,
                        8.0,
                        Color::from_rgba(200, 230, 255, 255),
                    );
                    draw_triangle(
                        vec2(x - 1.0, y + 10.0),
                        vec2(x + TILE_PX / 2.0, y - 2.0),
                        vec2(x + TILE_PX + 1.0, y + 10.0),
                        Color::from_rgba(48, 88, 168, 255),
                    );
                    draw_text("$", x + 14.0, y + 32.0, 16.0, C_ACCENT);
                }
                Tile::MartDoor => {
                    draw_rectangle(
                        x + 8.0,
                        y + 8.0,
                        TILE_PX - 16.0,
                        TILE_PX - 8.0,
                        Color::from_rgba(60, 44, 28, 255),
                    );
                    draw_circle(x + TILE_PX - 14.0, y + 22.0, 2.0, WHITE);
                }
                Tile::Floor => {
                    draw_rectangle_lines(
                        x + 1.0,
                        y + 1.0,
                        TILE_PX - 2.0,
                        TILE_PX - 2.0,
                        1.0,
                        Color::from_rgba(180, 160, 120, 80),
                    );
                }
                Tile::Flower => {
                    let bob = (t * 2.0 + tx as f32).sin() * 1.5;
                    draw_line(
                        x + 10.0,
                        y + TILE_PX - 4.0,
                        x + 10.0,
                        y + 16.0 + bob,
                        1.5,
                        Color::from_rgba(40, 120, 50, 255),
                    );
                    draw_circle(x + 10.0, y + 14.0 + bob, 4.5, PINK);
                    draw_circle(x + 10.0, y + 14.0 + bob, 2.0, YELLOW);
                    draw_line(
                        x + 22.0,
                        y + TILE_PX - 4.0,
                        x + 22.0,
                        y + 18.0 - bob,
                        1.5,
                        Color::from_rgba(40, 120, 50, 255),
                    );
                    draw_circle(
                        x + 22.0,
                        y + 16.0 - bob,
                        4.0,
                        Color::from_rgba(255, 120, 200, 255),
                    );
                    draw_circle(x + 22.0, y + 16.0 - bob, 1.8, WHITE);
                }
                Tile::Path => {
                    if (tx * 7 + ty * 3) % 4 == 0 {
                        draw_circle(x + 10.0, y + 18.0, 1.2, Color::from_rgba(150, 130, 90, 100));
                    }
                    draw_rectangle_lines(
                        x,
                        y,
                        TILE_PX,
                        TILE_PX,
                        0.6,
                        Color::from_rgba(0, 0, 0, 18),
                    );
                }
                Tile::Sand => {
                    for i in 0..3 {
                        let ox = x + 6.0 + i as f32 * 10.0;
                        let oy = y + 10.0 + ((tx + i) % 3) as f32 * 6.0;
                        draw_circle(ox, oy, 1.0, Color::from_rgba(180, 150, 100, 100));
                    }
                    draw_rectangle_lines(
                        x,
                        y,
                        TILE_PX,
                        TILE_PX,
                        0.5,
                        Color::from_rgba(160, 130, 80, 40),
                    );
                }
                Tile::Fence => {
                    draw_rectangle(
                        x + 4.0,
                        y + 10.0,
                        TILE_PX - 8.0,
                        4.0,
                        Color::from_rgba(140, 100, 55, 255),
                    );
                    draw_rectangle(
                        x + 4.0,
                        y + 22.0,
                        TILE_PX - 8.0,
                        4.0,
                        Color::from_rgba(140, 100, 55, 255),
                    );
                    draw_rectangle(
                        x + 8.0,
                        y + 6.0,
                        5.0,
                        26.0,
                        Color::from_rgba(110, 78, 42, 255),
                    );
                    draw_rectangle(
                        x + 26.0,
                        y + 6.0,
                        5.0,
                        26.0,
                        Color::from_rgba(110, 78, 42, 255),
                    );
                }
                Tile::Sign => {
                    // Subtle path under sign; post drawn by draw_world_props
                    draw_rectangle_lines(
                        x,
                        y,
                        TILE_PX,
                        TILE_PX,
                        0.6,
                        Color::from_rgba(0, 0, 0, 18),
                    );
                }
                _ => {}
            }
        }
    }
}

/// Draw signs and NPCs in world space.
pub fn draw_world_props(cam_x: f32, cam_y: f32) {
    let t = get_time() as f32;
    let sw = screen_width();
    let sh = screen_height();
    for prop in world_props() {
        let x = prop.tx as f32 * TILE_PX - cam_x;
        let y = prop.ty as f32 * TILE_PX - cam_y;
        if x + TILE_PX < -20.0 || y + TILE_PX < -20.0 || x > sw + 20.0 || y > sh + 20.0 {
            continue;
        }
        if prop.kind == PropKind::Sign {
            // Wooden signpost
            draw_rectangle(
                x + 17.0,
                y + 18.0,
                6.0,
                18.0,
                Color::from_rgba(110, 78, 42, 255),
            );
            draw_rectangle(
                x + 6.0,
                y + 6.0,
                28.0,
                18.0,
                Color::from_rgba(190, 150, 90, 255),
            );
            draw_rectangle_lines(
                x + 6.0,
                y + 6.0,
                28.0,
                18.0,
                1.5,
                Color::from_rgba(90, 60, 30, 255),
            );
            draw_line(
                x + 10.0,
                y + 12.0,
                x + 30.0,
                y + 12.0,
                1.2,
                Color::from_rgba(80, 50, 25, 180),
            );
            draw_line(
                x + 10.0,
                y + 17.0,
                x + 26.0,
                y + 17.0,
                1.2,
                Color::from_rgba(80, 50, 25, 140),
            );
        } else {
            // NPC person (simple sprite distinct from player)
            let bob = (t * 2.0 + prop.tx as f32).sin() * 1.2;
            let shirt = match prop.tx % 5 {
                0 => Color::from_rgba(220, 90, 120, 255),
                1 => Color::from_rgba(90, 180, 120, 255),
                2 => Color::from_rgba(200, 160, 60, 255),
                3 => Color::from_rgba(160, 100, 200, 255),
                _ => Color::from_rgba(100, 160, 220, 255),
            };
            draw_ellipse(
                x + TILE_PX / 2.0,
                y + TILE_PX - 4.0,
                12.0,
                4.0,
                0.0,
                Color::from_rgba(0, 0, 0, 55),
            );
            draw_rectangle(
                x + 12.0,
                y + 24.0 + bob,
                4.0,
                6.0,
                Color::from_rgba(50, 50, 70, 255),
            );
            draw_rectangle(
                x + 18.0,
                y + 24.0 + bob,
                4.0,
                6.0,
                Color::from_rgba(50, 50, 70, 255),
            );
            draw_rectangle(x + 10.0, y + 14.0 + bob, 14.0, 12.0, shirt);
            draw_circle(
                x + 17.0,
                y + 11.0 + bob,
                6.5,
                Color::from_rgba(255, 214, 176, 255),
            );
            // Hair / hat variety
            draw_rectangle(
                x + 11.0,
                y + 5.0 + bob,
                12.0,
                5.0,
                Color::from_rgba(60, 40, 30, 255),
            );
            // Idle ! cue when close (always subtle)
            if (t * 3.0).sin() > 0.3 {
                draw_text("!", x + 28.0, y + 8.0 + bob, 14.0, C_ACCENT);
            }
        }
    }
}

/// Lead Pokémon follower behind the player.
pub fn draw_follower(
    sprites: &SpriteCache,
    species_id: u16,
    player_px: f32,
    player_py: f32,
    facing: u8,
    cam_x: f32,
    cam_y: f32,
    walk_phase: f32,
) {
    // Offset behind player based on facing
    let (ox, oy) = match facing {
        1 => (18.0, 4.0),  // facing left -> follower right
        2 => (-18.0, 4.0), // facing right -> follower left
        3 => (2.0, 16.0),  // facing up -> follower below
        _ => (2.0, -14.0), // facing down -> follower above/behind
    };
    let bob = (walk_phase * 10.0).sin().abs() * 2.0;
    let x = player_px + ox - cam_x;
    let y = player_py + oy - cam_y + bob;
    let size = 28.0;
    draw_ellipse(
        x + size * 0.5,
        y + size * 0.85,
        size * 0.32,
        size * 0.1,
        0.0,
        Color::from_rgba(0, 0, 0, 45),
    );
    draw_pokemon_sprite(sprites, species_id, x, y, size, false, 0.0);
}

pub fn draw_player(px: f32, py: f32, cam_x: f32, cam_y: f32, facing: u8, walk_phase: f32) {
    let x = px - cam_x;
    let y = py - cam_y;
    let bob = (walk_phase * 12.0).sin().abs() * 2.2;
    let leg = (walk_phase * 12.0).sin() * 2.0;

    draw_ellipse(
        x + TILE_PX / 2.0,
        y + TILE_PX - 3.0,
        13.0,
        4.5,
        0.0,
        Color::from_rgba(0, 0, 0, 70),
    );

    draw_rectangle(
        x + 11.0,
        y + 24.0 + bob,
        4.0,
        6.0 + leg.max(0.0),
        Color::from_rgba(50, 70, 140, 255),
    );
    draw_rectangle(
        x + 17.0,
        y + 24.0 + bob,
        4.0,
        6.0 - leg.min(0.0),
        Color::from_rgba(50, 70, 140, 255),
    );

    draw_rectangle(
        x + 9.0,
        y + 13.0 + bob,
        14.0,
        13.0,
        Color::from_rgba(62, 122, 220, 255),
    );
    draw_rectangle(
        x + 9.0,
        y + 13.0 + bob,
        14.0,
        4.0,
        Color::from_rgba(48, 98, 190, 255),
    );

    if facing != 0 {
        draw_rectangle(
            x + 7.0,
            y + 14.0 + bob,
            5.0,
            9.0,
            Color::from_rgba(180, 70, 50, 255),
        );
    }

    draw_circle(
        x + 16.0,
        y + 10.0 + bob,
        7.2,
        Color::from_rgba(255, 214, 176, 255),
    );

    draw_rectangle(
        x + 9.0,
        y + 4.0 + bob,
        14.0,
        5.0,
        Color::from_rgba(220, 48, 48, 255),
    );
    draw_rectangle(
        x + 11.0,
        y + 1.5 + bob,
        10.0,
        4.0,
        Color::from_rgba(200, 36, 36, 255),
    );
    draw_rectangle(
        x + 16.0,
        y + 6.5 + bob,
        9.0,
        2.5,
        Color::from_rgba(220, 48, 48, 255),
    );

    let eye_off = match facing {
        1 => (-2.5, 0.5),
        2 => (2.5, 0.5),
        3 => (0.0, -0.5),
        _ => (0.0, 1.2),
    };
    draw_circle(
        x + 13.5 + eye_off.0,
        y + 10.0 + bob + eye_off.1,
        1.6,
        Color::from_rgba(30, 30, 40, 255),
    );
    draw_circle(
        x + 18.5 + eye_off.0,
        y + 10.0 + bob + eye_off.1,
        1.6,
        Color::from_rgba(30, 30, 40, 255),
    );
    draw_circle(x + 14.0 + eye_off.0, y + 9.5 + bob + eye_off.1, 0.6, WHITE);
    draw_circle(x + 19.0 + eye_off.0, y + 9.5 + bob + eye_off.1, 0.6, WHITE);
}

pub fn draw_hud_bar(money: u32, balls: u32, potions: u32, party_count: usize, area: &str) {
    let w = screen_width();
    draw_rectangle(0.0, 0.0, w, 40.0, Color::new(0.05, 0.07, 0.13, 0.92));
    draw_rectangle(0.0, 38.0, w, 2.0, C_ACCENT);
    draw_rectangle(0.0, 0.0, w, 2.0, Color::new(1.0, 1.0, 1.0, 0.08));

    draw_panel(10.0, 7.0, 108.0, 26.0, true);
    draw_circle(26.0, 20.0, 7.0, Color::from_rgba(240, 200, 60, 255));
    draw_text("$", 22.0, 25.0, 14.0, Color::from_rgba(60, 40, 10, 255));
    draw_text(&format!("{money}"), 38.0, 25.0, 16.0, C_ACCENT);

    draw_panel(128.0, 7.0, 96.0, 26.0, false);
    draw_circle(144.0, 20.0, 7.0, Color::from_rgba(220, 70, 70, 255));
    draw_circle(144.0, 20.0, 4.0, Color::from_rgba(255, 220, 220, 255));
    draw_text(&format!("×{balls}"), 156.0, 25.0, 15.0, C_TEXT);

    draw_panel(234.0, 7.0, 96.0, 26.0, false);
    draw_rectangle(246.0, 14.0, 10.0, 12.0, Color::from_rgba(90, 200, 110, 255));
    draw_rectangle(248.0, 12.0, 6.0, 4.0, Color::from_rgba(70, 160, 90, 255));
    draw_text(&format!("×{potions}"), 262.0, 25.0, 15.0, C_TEXT);

    draw_panel(340.0, 7.0, 108.0, 26.0, false);
    draw_text(&format!("Party {party_count}/6"), 352.0, 25.0, 14.0, C_TEXT);

    let aw = measure_text(area, None, 15, 1.0).width + 28.0;
    let ax = (w - aw - 260.0).max(460.0);
    draw_panel(ax, 7.0, aw, 26.0, false);
    draw_text(area, ax + 14.0, 25.0, 15.0, C_ACCENT);

    draw_text(
        "[E/H] Talk/Heal/Shop   [P] Party   [Esc] Menu",
        w - 298.0,
        25.0,
        13.0,
        C_TEXT_DIM,
    );
}

pub fn draw_toast(msg: &str, timer: f32) {
    if timer <= 0.0 || msg.is_empty() {
        return;
    }
    let alpha = timer.min(1.0);
    let slide = (1.0 - alpha) * 14.0;
    let tw = measure_text(msg, None, 18, 1.0).width + 56.0;
    let x = (screen_width() - tw) / 2.0;
    let y = screen_height() - 92.0 + slide;
    let a = (alpha * 255.0) as u8;
    draw_rectangle(
        x + 4.0,
        y + 5.0,
        tw,
        42.0,
        Color::from_rgba(0, 0, 0, (a / 2).max(24)),
    );
    draw_rectangle(x, y, tw, 42.0, Color::from_rgba(14, 20, 38, a.min(240)));
    draw_rectangle(x, y, 5.0, 42.0, Color::from_rgba(240, 200, 60, a));
    draw_rectangle_lines(
        x,
        y,
        tw,
        42.0,
        1.5,
        Color::from_rgba(240, 200, 60, (a as f32 * 0.85) as u8),
    );
    draw_text(
        msg,
        x + 28.0,
        y + 28.0,
        18.0,
        Color::from_rgba(255, 255, 255, a),
    );
}

/// Word-wrap at spaces when possible (up to `max_cols` per row, max `max_rows`).
pub(crate) fn wrap_dialogue_line(line: &str, max_cols: usize, max_rows: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut rest = line.trim();
    while !rest.is_empty() && rows.len() < max_rows {
        if rest.chars().count() <= max_cols {
            rows.push(rest.to_string());
            break;
        }
        let mut end_byte = rest.len();
        let mut last_space = None;
        for (i, ch) in rest.char_indices() {
            if i > 0 && rest[..i].chars().count() >= max_cols {
                end_byte = i;
                break;
            }
            if ch == ' ' {
                last_space = Some(i);
            }
        }
        let split = last_space
            .filter(|&sp| rest[..sp].chars().count() > max_cols / 3)
            .unwrap_or(end_byte)
            .max(1)
            .min(rest.len());
        rows.push(rest[..split].trim_end().to_string());
        rest = rest[split..].trim_start();
    }
    if rows.is_empty() {
        rows.push(String::new());
    }
    rows
}

/// Classic RPG dialogue box (advance with Enter/Space/E/Esc).
pub fn draw_dialogue_box(prop: &WorldProp, line_idx: usize) {
    let w = screen_width();
    let h = screen_height();
    draw_modal_dim(0.28);
    let bw = w - 80.0;
    let bh = 148.0;
    let bx = 40.0;
    let by = h - bh - 44.0;
    draw_panel(bx, by, bw, bh, true);
    draw_rectangle(bx + 8.0, by + 8.0, bw - 16.0, 4.0, C_ACCENT);

    draw_title_text(
        &format!("{} — {}", prop.kind.label(), prop.title),
        bx + 20.0,
        by + 34.0,
        20.0,
    );

    let line = prop.lines.get(line_idx).copied().unwrap_or("");
    for (row, slice) in wrap_dialogue_line(line, 68, 3).iter().enumerate() {
        draw_text(
            slice,
            bx + 24.0,
            by + 64.0 + row as f32 * 22.0,
            18.0,
            C_TEXT,
        );
    }

    let more = line_idx + 1 < prop.lines.len();
    let hint = if more {
        "Enter / Space / E  —  next"
    } else {
        "Enter / Space / E / Esc  —  close"
    };
    let pulse = ((get_time() as f32 * 4.0).sin() * 0.5 + 0.5) * 0.25 + 0.75;
    draw_text(
        hint,
        bx + bw - 250.0,
        by + bh - 16.0,
        13.0,
        Color::new(C_ACCENT.r, C_ACCENT.g, C_ACCENT.b, pulse),
    );
    draw_text(
        &format!("{}/{}", line_idx + 1, prop.lines.len().max(1)),
        bx + 24.0,
        by + bh - 16.0,
        13.0,
        C_TEXT_DIM,
    );
}

#[cfg(test)]
mod tests {
    use super::wrap_dialogue_line;

    #[test]
    fn wrap_prefers_spaces() {
        let rows = wrap_dialogue_line("hello world from the dialogue system", 12, 3);
        assert!(rows.len() >= 2);
        assert!(rows[0].len() <= 12 || rows[0].chars().count() <= 12);
        // Should not start with a leading space
        for r in &rows {
            assert!(!r.starts_with(' '));
        }
    }

    #[test]
    fn wrap_short_single_row() {
        let rows = wrap_dialogue_line("short", 68, 3);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], "short");
    }
}
