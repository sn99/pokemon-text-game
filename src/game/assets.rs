//! Texture loading for Pokemon sprites (lazy + progress for starters).

use std::collections::HashMap;
use std::path::Path;

use macroquad::prelude::*;

use super::theme::{
    draw_gradient_bg, draw_panel, draw_title_text, draw_vignette, C_ACCENT, C_TEXT_DIM,
};

/// Priority IDs shown on title/starter — load first so menus look good quickly.
const PRIORITY_IDS: &[u16] = &[1, 3, 4, 6, 7, 9, 25, 150];

pub struct SpriteCache {
    front: HashMap<u16, Texture2D>,
    back: HashMap<u16, Texture2D>,
    placeholder: Texture2D,
}

impl SpriteCache {
    fn make_placeholder() -> Texture2D {
        let placeholder = Texture2D::from_rgba8(
            2,
            2,
            &[
                200, 80, 80, 255, 80, 200, 80, 255, 80, 80, 200, 255, 200, 200, 80, 255,
            ],
        );
        placeholder.set_filter(FilterMode::Nearest);
        placeholder
    }

    async fn load_one_front(id: u16, front: &mut HashMap<u16, Texture2D>) {
        if front.contains_key(&id) {
            return;
        }
        let fp = format!("assets/sprites/pokemon/{id}.png");
        if Path::new(&fp).exists() {
            if let Ok(tex) = load_texture(&fp).await {
                tex.set_filter(FilterMode::Nearest);
                front.insert(id, tex);
            }
        }
    }

    async fn load_one_back(id: u16, back: &mut HashMap<u16, Texture2D>) {
        if back.contains_key(&id) {
            return;
        }
        let bp = format!("assets/sprites/pokemon/back/{id}.png");
        if Path::new(&bp).exists() {
            if let Ok(tex) = load_texture(&bp).await {
                tex.set_filter(FilterMode::Nearest);
                back.insert(id, tex);
            }
        }
    }

    /// Fast start: priority sprites + loading bar, then rest of dex.
    pub async fn load_with_progress() -> Self {
        let placeholder = Self::make_placeholder();
        let mut front = HashMap::new();
        let mut back = HashMap::new();
        let total = 151u16;

        // Phase 1 — starters / title mascots
        for (i, &id) in PRIORITY_IDS.iter().enumerate() {
            Self::load_one_front(id, &mut front).await;
            Self::load_one_back(id, &mut back).await;
            draw_loading_frame(
                (i + 1) as f32 / (PRIORITY_IDS.len() + total as usize) as f32,
                id,
            );
            next_frame().await;
        }

        // Phase 2 — full catalog
        for id in 1u16..=total {
            Self::load_one_front(id, &mut front).await;
            Self::load_one_back(id, &mut back).await;
            if id % 5 == 0 || id == total {
                let base = PRIORITY_IDS.len() as f32;
                let p = (base + id as f32) / (base + total as f32);
                draw_loading_frame(p, id);
                next_frame().await;
            }
        }

        Self {
            front,
            back,
            placeholder,
        }
    }

    pub fn front(&self, id: u16) -> &Texture2D {
        self.front.get(&id).unwrap_or(&self.placeholder)
    }

    pub fn back(&self, id: u16) -> &Texture2D {
        self.back
            .get(&id)
            .or_else(|| self.front.get(&id))
            .unwrap_or(&self.placeholder)
    }
}

fn draw_loading_frame(progress: f32, id: u16) {
    let w = screen_width();
    let h = screen_height();
    draw_gradient_bg(
        Color::new(0.06, 0.09, 0.18, 1.0),
        Color::new(0.10, 0.18, 0.32, 1.0),
        24,
    );
    draw_vignette();
    let t = get_time() as f32;
    let bob = (t * 2.0).sin() * 4.0;
    draw_title_text("POKÉMON 2D", w / 2.0 - 140.0, h * 0.30 + bob, 48.0);
    draw_text(
        "Loading sprites…",
        w / 2.0 - 70.0,
        h * 0.39,
        20.0,
        C_TEXT_DIM,
    );

    let bar_w = 420.0;
    let bar_h = 22.0;
    let bx = w / 2.0 - bar_w / 2.0;
    let by = h * 0.48;
    draw_panel(bx - 8.0, by - 8.0, bar_w + 16.0, bar_h + 16.0, true);
    draw_rectangle(bx, by, bar_w, bar_h, Color::from_rgba(20, 24, 36, 255));
    let fill = bar_w * progress.clamp(0.0, 1.0);
    draw_rectangle(bx, by, fill, bar_h, C_ACCENT);
    // Shine on progress fill
    if fill > 8.0 {
        draw_rectangle(bx, by, fill, 6.0, Color::new(1.0, 1.0, 1.0, 0.18));
    }
    draw_rectangle(bx, by, fill, bar_h * 0.4, Color::new(1.0, 1.0, 1.0, 0.25));
    draw_rectangle_lines(bx, by, bar_w, bar_h, 2.0, WHITE);
    draw_text(
        &format!("{}%  ·  #{id}/151", (progress * 100.0) as u32),
        w / 2.0 - 55.0,
        by + 48.0,
        16.0,
        C_TEXT_DIM,
    );
    draw_text(
        "Fan-made · Not affiliated with Nintendo",
        w / 2.0 - 140.0,
        h * 0.72,
        14.0,
        C_TEXT_DIM,
    );
}
