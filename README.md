# Pokemon Text Game

A **full terminal Pokemon battler** in Rust — ratatui UI, type-effective combat, ASCII sprites, adventure progression, and PokeAPI-shaped data.

> Fan-made. Not affiliated with Nintendo / Game Freak / Creatures.

**Version 2.0.0**

## Features

- **Adventure mode** — starters, wild grass, XP, money, heal center, save/load
- **Classic battles** — PvC / PvP via `resources/pokemons.json`
- **Type chart** — 18 types, dual-type mult, STAB, accuracy, crits, PP
- **ASCII sprites** — side-by-side battlefield (toggle `s` in battle)
- **Species dex** — browse stats + art
- **Team editor** — create/edit/delete exhibition Pokemon
- **Modular** — library + TUI, 50+ unit/integration tests

## Quick start

```bash
# Linux (Debian/Ubuntu) audio deps
sudo apt install pkg-config libasound2-dev

cargo run          # from repo root
cargo test
cargo build --release
```

Saves: OS app-data dir `pokemon-text-game/save.json`.

## Controls

| Key | Action |
|-----|--------|
| ↑/↓ or j/k | Navigate |
| Enter / Space | Confirm / attack |
| s | Toggle sprites (battle) |
| q / Esc | Back |
| Ctrl+C | Quit |

## PokeAPI data

```bash
python3 scripts/import_pokeapi.py --pokeapi-root /path/to/pokeapi -o resources/data/species.json
# or
python3 scripts/import_pokeapi.py --live --max-id 151 -o resources/data/species.json
```

See [docs/DATA_AND_POKEAPI.md](docs/DATA_AND_POKEAPI.md).

## Docs

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/GAMEPLAY.md](docs/GAMEPLAY.md)
- [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)
- [docs/TESTING.md](docs/TESTING.md)
- [docs/CHANGELOG.md](docs/CHANGELOG.md)

## Layout

```
src/app/        TUI
src/pokemon/    Types, stats, moves, species
src/battle/     Engine + encounters
src/ascii/      Sprites
src/audio/      Music
src/data/       JSON / PokeAPI
src/save/       Adventure saves
resources/      Assets
scripts/        Import helpers
```

## License

MIT — [LICENSE.md](LICENSE.md)

## Credits

sn99 · ratatui · rodio · PokeAPI (data inspiration)
