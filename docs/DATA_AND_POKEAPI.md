# Data & PokeAPI — real pipeline

This game is built against the official PokeAPI **data/v2** dump and **sprites** repo, not hand-waved stubs.

## Repos used

| Source | URL |
|--------|-----|
| Stats / types / moves / flavor text | https://github.com/PokeAPI/pokeapi/tree/master/data/v2 |
| Front sprites (PNG → ASCII) | https://github.com/PokeAPI/sprites |

## One-shot rebuild (recommended)

```bash
# 1. Clone upstream data (already done if you ran the agent once)
git clone --depth 1 https://github.com/PokeAPI/pokeapi.git /tmp/pokeapi
git clone --depth 1 --filter=blob:none --sparse https://github.com/PokeAPI/sprites.git /tmp/pokeapi-sprites
cd /tmp/pokeapi-sprites && git sparse-checkout set sprites/pokemon

# 2. Build species.json (1025) + moves.json (937) + pokemons.json
cd /path/to/pokemon-text-game
python3 scripts/build_game_data.py \
  --pokeapi-root /tmp/pokeapi \
  --sprites-root /tmp/pokeapi-sprites \
  --max-species 1025 \
  --out-dir resources

# 3. High-quality ASCII sprites via chafa (block symbols)
python3 - <<'PY'
# (or re-run the chafa batch in scripts/sprites_to_ascii.py after fixing --colors)
# Current best output is produced by the chafa block pipeline in the agent session.
PY
```

## What gets written

| File | Contents |
|------|----------|
| `resources/data/species.json` | ~1025 species: names, types, base stats, flavor text, capture rate, default move ids |
| `resources/data/moves.json` | ~937 moves: type, category, power, accuracy, PP, English flavor |
| `resources/sprites/ascii/NNN.txt` | 1025 ASCII frames from PokeAPI PNGs (chafa block art) |
| `resources/pokemons.json` | Classic team seeded from real dex entries |

## Runtime loading

`pokemon::db` (`src/pokemon/db.rs`) loads JSON **once** into `OnceLock` caches:

- `all_species()` / `species_by_id` / `species_by_name`
- `all_moves()` / `move_by_id` / `move_by_name`

If JSON is missing, falls back to the small compiled `builtin_*` catalogues so tests still run in CI without the multi-MB data.

The TUI header shows live counts, e.g. `1025 sp / 937 moves`.

## Audio

Procedural MP3 placeholders (ffmpeg sine layers) live in `resources/audio/`:

- `track.mp3` — title/menu
- `battle.mp3` — battle
- `victory.mp3` — win
- `pokedex.mp3` — dex/menus

These are **original/generated**, not ripped game music. Swap in your own licensed tracks for production/sale.

## Legal

Pokemon names/stats/sprites are IP of Nintendo/Game Freak/Creatures. PokeAPI redistributes data for community use. This project is a fan engine; commercial distribution needs your own legal review and asset licensing.
