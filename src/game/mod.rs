//! 2D game loop: overworld, battles, menus (macroquad).

mod assets;
mod battle_ui;
mod overworld;
mod state;
mod theme;

pub use state::{run_game, Game};
