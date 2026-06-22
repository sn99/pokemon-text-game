//! Shared utility helpers (parsing, indices, paths).

/// Parse a trimmed string as i64 without panicking.
pub fn parse_i64(s: &str) -> Result<i64, String> {
    s.trim()
        .parse::<i64>()
        .map_err(|_| format!("'{}' is not a valid number", s.trim()))
}

/// Validate a 1-based menu/index choice.
pub fn is_valid_choice(choice: i64, bounds: i64) -> bool {
    choice >= 1 && choice <= bounds
}

/// Split a space-separated moves string into non-empty move names.
pub fn parse_moves(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter(|m| !m.is_empty())
        .map(|m| m.to_string())
        .collect()
}

/// Clamp a list index selection after up/down navigation.
pub fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        index.min(len - 1)
    }
}

/// Move selection index with wrap-around (len > 0).
pub fn move_selection(current: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as isize;
    let next = current as isize + delta;
    (((next % len_i) + len_i) % len_i) as usize
}

/// Resolve a path relative to the executable / CWD, preferring resources next to CWD.
pub fn resource_path(relative: &str) -> std::path::PathBuf {
    let candidates = [
        std::path::PathBuf::from(relative),
        std::path::PathBuf::from("resources").join(relative.trim_start_matches("resources/")),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    std::path::PathBuf::from(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_choice_within_bounds() {
        assert!(is_valid_choice(1, 3));
        assert!(is_valid_choice(3, 3));
    }

    #[test]
    fn invalid_choice_outside_bounds() {
        assert!(!is_valid_choice(0, 3));
        assert!(!is_valid_choice(4, 3));
    }

    #[test]
    fn parse_i64_accepts_numbers() {
        assert_eq!(parse_i64("42").unwrap(), 42);
        assert_eq!(parse_i64("  -7 \n").unwrap(), -7);
    }

    #[test]
    fn parse_i64_rejects_garbage() {
        assert!(parse_i64("none").is_err());
        assert!(parse_i64("").is_err());
    }

    #[test]
    fn parse_moves_splits_and_filters() {
        assert_eq!(parse_moves("  IronTail   Tackle  "), vec!["IronTail", "Tackle"]);
        assert!(parse_moves("   ").is_empty());
    }

    #[test]
    fn move_selection_wraps() {
        assert_eq!(move_selection(0, -1, 3), 2);
        assert_eq!(move_selection(2, 1, 3), 0);
        assert_eq!(move_selection(0, 1, 0), 0);
    }

    #[test]
    fn clamp_index_handles_empty() {
        assert_eq!(clamp_index(5, 0), 0);
        assert_eq!(clamp_index(5, 3), 2);
    }
}
