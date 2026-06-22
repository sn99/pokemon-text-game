# Pokemon 2D

A **simplified 2D Pokemon adventure** in Rust — walk an overworld, fight wild Pokemon with real sprites, catch, level up, and evolve.

> Fan-made. Not affiliated with Nintendo / Game Freak / Creatures.

**Version 3.2.0** — 2D macroquad adventure (not terminal UI).

## What's in (kept deliberately simple)

| Feature | Notes |
|--------|--------|
| Overworld | Town, routes, tall grass, pond, sand, fences, smooth camera |
| Starters | Bulbasaur / Charmander / Squirtle |
| Wild battles | Types, STAB, PP, crits, speed order, switch |
| Catch | Balls with on-screen catch % estimate |
| Mart | Blue shop — buy balls ($100) & potions ($80) |
| Center | Red building — step on door or **E** to heal |
| Party | Up to 6, potions, level-up & evolutions |
| Save | Autosave on milestones; Pause → Save |
| Signs & NPCs | Press **E** to read tips / encounter lists |
| Pokédex | Pause → Pokédex; seen/caught from battles |
| Follower | Lead Pokémon sprite trails you in overworld |

**Removed** vs old TUI v2: Elite Four, gyms, achievements, daily challenges, team editor, ASCII UI, etc.

## Quick start

```bash
# From repo root (sprites already under assets/sprites/pokemon/)
cargo run

# Release build
cargo build --release
./target/release/pokemon-text-game

cargo test
```

Saves: OS app-data dir `pokemon-text-game/save_v3.json`.

## Controls

| Key | Action |
|-----|--------|
| Arrow keys / WASD | Move / menus |
| Enter / Space | Confirm |
| E / H | Talk (signs/NPCs), heal at Center, or open Mart |
| P | Party (Enter = use potion out of battle) |
| Esc | Pause / back |
| Battle | Fight · Bag · Switch · Run; moves **1–4** quick-select |

## Sprites

Official-style sprites from [PokeAPI sprites](https://github.com/PokeAPI/sprites) (Gen 1 front + back, ids 1–151) under `assets/sprites/pokemon/`.

Re-download if needed:

```bash
python3 -c "
import os, urllib.request, concurrent.futures
base='https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon'
for kind, sub in [('f',''),('b','/back')]:
  os.makedirs(f'assets/sprites/pokemon{sub}', exist_ok=True)
  def dl(i):
    url=f'{base}{sub}/{i}.png'
    path=f'assets/sprites/pokemon{sub}/{i}.png'
    try: urllib.request.urlretrieve(url, path)
    except: pass
  with concurrent.futures.ThreadPoolExecutor(16) as ex:
    list(ex.map(dl, range(1,152)))
print('done')
"
```

Game data (species/moves) still comes from `resources/data/` (PokeAPI-shaped JSON).

## Project layout

```
src/game/       2D loop (overworld, battle UI, assets)
src/pokemon/    Types, stats, moves, species
src/battle/     Damage engine
src/world/      Map + evolutions + wild pools
src/save/       Minimal save format
assets/         PNG sprites
resources/data/ Species & moves JSON
```

## License

MIT — [LICENSE.md](LICENSE.md)

## Credits

sn99 · macroquad · PokeAPI (sprites & data inspiration)