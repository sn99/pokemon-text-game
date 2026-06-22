//! ASCII / color-block sprite rendering for terminal battles.

pub mod sprites;
pub mod color_sprite;

pub use color_sprite::{
    color_sprite_count, color_sprite_for_species, load_color_sprite, ColorCell, ColorSprite,
};
pub use sprites::{
    battle_frame, flip_horizontal, generate_procedural_sprite, sprite_for_species, SPRITE_HEIGHT,
    SPRITE_WIDTH,
};
