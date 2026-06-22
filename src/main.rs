/* MIT — sn99 — Pokemon 2D */

use macroquad::prelude::*;

use pokemon_text_game::game::run_game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Pokémon 2D — v3.2".to_owned(),
        window_width: 1024,
        window_height: 680,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    run_game().await;
}
