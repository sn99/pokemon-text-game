# Assets & Attribution

Fan-made project — **not affiliated** with Nintendo, Game Freak, or Creatures Inc.

## Pokémon sprites (`assets/sprites/pokemon/`)

| Source | License / notes |
|--------|-----------------|
| [PokeAPI sprites](https://github.com/PokeAPI/sprites) | Community sprite mirror; official-style Gen 1 front & back (ids 1–151). Used for battles, title, party, follower, Pokédex. |

Re-download helper is documented in the root `README.md`.

## Game data (`resources/data/`)

| File | Notes |
|------|-------|
| `species.json`, `moves.json` | PokeAPI-shaped catalogues (see `docs/DATA_AND_POKEAPI.md`). |

## Audio (`resources/audio/`)

Optional BGM tracks; missing files degrade gracefully. See `resources/audio/README.md`.

## Procedural / code-drawn art

Tiles, player/NPC avatars, signs, UI chrome, HP bars, and battle backdrops are drawn in Rust (`src/game/overworld.rs`, `battle_ui.rs`, `theme.rs`) — no third-party tile assets required.
