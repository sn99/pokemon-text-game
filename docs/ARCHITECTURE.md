# Architecture

Pokemon 2D v3 is a **library crate** (`src/lib.rs`) plus a thin **macroquad binary** (`src/main.rs`).

> Note: older branches/docs mentioned ratatui/`src/app/` terminal UI. Current runtime is **macroquad 2D** in `src/game/`.

## Layout

```
src/
  lib.rs           Public API
  main.rs          Window + run_game()
  game/            2D loop: overworld, battle UI, assets, theme, state machine
  pokemon/         Domain: types, stats, moves, species, instances
  battle/          Damage engine, wild encounters
  world/           Map tiles, props (signs/NPCs), evolutions, areas
  save/            Adventure saves, inventory, dex, normalize/load limits
  data/            Team/species JSON I/O, PokeAPI adapters
  ascii/           Legacy/procedural sprites (optional)
  audio/           Optional music helpers
resources/
  data/species.json, moves.json
  audio/           Optional BGM
assets/sprites/pokemon/  Gen1 front+back PNGs (PokeAPI sprites mirror)
scripts/           Import & asset helpers
docs/              Human documentation
tests/             Integration tests
```

## Screens (`src/game/state.rs`)

Title → Starter → Overworld → Battle | Party | Pause | Shop | Dialogue | Pokédex

## Extension points

| Goal | Where |
|------|--------|
| New species | `pokemon/species.rs` or import JSON |
| New moves | `pokemon/moves.rs` |
| Type chart | `pokemon/types.rs` |
| Map / props | `world/mod.rs` (`build_map`, `world_props`) |
| UI screens | `game/state.rs` + `overworld.rs` / `battle_ui.rs` |
| Sprites | `assets/sprites/pokemon/{id}.png` |

## Principles

- Game logic unit-testable without a terminal (`world`, `save`, `battle`, `pokemon`).
- Missing audio/save/sprites degrade gracefully.
- Saves are size-capped and normalized on load/write (`save::SaveGame::normalize`).
