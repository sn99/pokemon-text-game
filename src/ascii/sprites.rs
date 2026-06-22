//! Procedural and file-backed ASCII sprites for battle display.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::pokemon::types::ElementType;

/// Width/height of the standard battle sprite frame.
pub const SPRITE_WIDTH: usize = 34;
pub const SPRITE_HEIGHT: usize = 16;

static SPRITE_CACHE: OnceLock<HashMap<u16, Vec<String>>> = OnceLock::new();

/// Load optional overrides from `resources/sprites/ascii/{id}.txt`.
pub fn load_sprite_overrides(dir: &Path) -> HashMap<u16, Vec<String>> {
    let mut map = HashMap::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return map;
    };
    for ent in entries.flatten() {
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if let Ok(id) = stem.parse::<u16>() {
            if let Ok(content) = fs::read_to_string(&path) {
                let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                if !lines.is_empty() {
                    map.insert(id, lines);
                }
            }
        }
    }
    map
}

fn cache() -> &'static HashMap<u16, Vec<String>> {
    SPRITE_CACHE.get_or_init(|| {
        let dir = Path::new("resources/sprites/ascii");
        load_sprite_overrides(dir)
    })
}

/// Get ASCII art lines for a species id (file override or procedural).
pub fn sprite_for_species(id: u16, primary_type: ElementType) -> Vec<String> {
    if let Some(lines) = cache().get(&id) {
        return pad_sprite(lines.clone());
    }
    pad_sprite(generate_procedural_sprite(id, primary_type))
}

fn pad_sprite(mut lines: Vec<String>) -> Vec<String> {
    // Keep file-backed sprites at their natural size (chafa output), only pad procedural ones.
    let target_h = lines.len().max(SPRITE_HEIGHT.min(lines.len().max(8)));
    while lines.len() < target_h.min(SPRITE_HEIGHT) {
        lines.push(String::new());
    }
    if lines.len() > SPRITE_HEIGHT + 4 {
        lines.truncate(SPRITE_HEIGHT + 4);
    }
    let max_w = lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(SPRITE_WIDTH)
        .max(SPRITE_WIDTH.min(34));
    for line in &mut lines {
        let n = line.chars().count();
        if n < max_w {
            line.push_str(&" ".repeat(max_w - n));
        }
    }
    lines
}

/// Deterministic pseudo-art shaped by type and species id.
pub fn generate_procedural_sprite(id: u16, t: ElementType) -> Vec<String> {
    let seed = id as usize;
    let (body, ear, eye, accent) = type_chars(t);

    // Build a simple creature silhouette
    let mut lines = vec![String::new(); SPRITE_HEIGHT];

    // Head / ears row
    lines[1] = format!("    {ear}    {ear}         ");
    lines[2] = format!("   {body}{body}{body}{body}{body}{body}        ");
    lines[3] = format!("  {body} {eye}  {eye} {body}       ");
    lines[4] = format!("  {body}  {accent}{accent}  {body}       ");
    lines[5] = format!("   {body}{body}{body}{body}{body}{body}        ");

    // Body / legs vary slightly by seed
    let leg = if seed % 3 == 0 { " /\\ " } else if seed % 3 == 1 { " || " } else { " /\\ " };
    lines[6] = format!("   {body}    {body}        ");
    lines[7] = format!("  {leg}  {leg}      ");
    lines[8] = format!("  /  \\  /  \\      ");

    // Tail / wing accent for flying/dragon/fire
    if matches!(t, ElementType::Flying | ElementType::Dragon | ElementType::Fire) {
        lines[2] = format!(" ~ {body}{body}{body}{body}{body}{body} ~      ");
        lines[5] = format!("  ~{body}{body}{body}{body}{body}{body}~       ");
    }
    if matches!(t, ElementType::Water | ElementType::Ice) {
        lines[8] = format!("  ~~~~  ~~~~      ");
    }
    if matches!(t, ElementType::Ghost) {
        lines[7] = format!("  |  |  |  |      ");
        lines[8] = format!("  v  v  v  v      ");
    }
    if matches!(t, ElementType::Electric) {
        lines[0] = "      *  *          ".to_string();
        lines[1] = format!("   * {ear}  {ear} *       ");
    }

    // Species number watermark in corner (tiny)
    let tag = format!("#{id:03}");
    if let Some(last) = lines.last_mut() {
        let mut chs: Vec<char> = last.chars().collect();
        while chs.len() < SPRITE_WIDTH {
            chs.push(' ');
        }
        for (i, c) in tag.chars().enumerate() {
            if i < chs.len() {
                chs[i] = c;
            }
        }
        *last = chs.into_iter().collect();
    }

    lines
}

fn type_chars(t: ElementType) -> (char, char, char, char) {
    // (body, ear, eye, accent)
    match t {
        ElementType::Fire => ('#', '^', 'o', '~'),
        ElementType::Water => ('O', '.', '@', '~'),
        ElementType::Grass => ('*', 'v', 'o', '='),
        ElementType::Electric => ('H', '^', '*', '!'),
        ElementType::Psychic => ('@', '~', '0', '*'),
        ElementType::Ghost => ('8', '^', 'x', 'o'),
        ElementType::Dragon => ('W', 'M', 'O', '='),
        ElementType::Ice => ('A', '^', 'o', '*'),
        ElementType::Fighting => ('M', 'r', '>', 'o'),
        ElementType::Poison => ('G', 'n', 'x', '~'),
        ElementType::Ground => ('m', '.', 'o', '-'),
        ElementType::Flying => ('v', '^', 'o', '~'),
        ElementType::Bug => ('o', '^', '.', '='),
        ElementType::Rock => ('A', '^', 'o', '='),
        ElementType::Dark => ('#', '^', '-', 'x'),
        ElementType::Steel => ('H', '|', 'o', '='),
        ElementType::Fairy => ('*', '^', 'o', '~'),
        ElementType::Normal => ('O', '^', 'o', '-'),
    }
}

/// Compose a side-by-side battle frame: player (left) vs foe (right).
pub fn battle_frame(
    player_sprite: &[String],
    foe_sprite: &[String],
    player_name: &str,
    foe_name: &str,
) -> Vec<String> {
    let gap = "    VS    ";
    let mut out = Vec::new();
    out.push(format!(
        "{:<width$}{}{:>width$}",
        format!("[{player_name}]"),
        gap,
        format!("[{foe_name}]"),
        width = SPRITE_WIDTH
    ));
    let h = player_sprite.len().max(foe_sprite.len());
    for i in 0..h {
        let l = player_sprite.get(i).map(|s| s.as_str()).unwrap_or("");
        let r = foe_sprite.get(i).map(|s| s.as_str()).unwrap_or("");
        out.push(format!("{l}{gap}{r}"));
    }
    out
}

/// Flip a sprite horizontally (simple char-level reverse) for the player side.
pub fn flip_horizontal(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|l| l.chars().rev().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_has_reasonable_dimensions() {
        let s = sprite_for_species(25, ElementType::Electric);
        assert!(s.len() >= 6);
        assert!(s.iter().any(|l| l.trim().len() > 2));
    }

    #[test]
    fn battle_frame_joins() {
        let a = sprite_for_species(25, ElementType::Electric);
        let b = sprite_for_species(4, ElementType::Fire);
        let frame = battle_frame(&a, &b, "Pikachu", "Charmander");
        assert!(frame.len() >= 2);
        assert!(frame[0].contains("Pikachu"));
        assert!(frame[0].contains("Charmander"));
    }

    #[test]
    fn flip_reverses() {
        let lines = vec!["abc".into(), "12".into()];
        let f = flip_horizontal(&lines);
        assert_eq!(f[0], "cba");
    }
}
