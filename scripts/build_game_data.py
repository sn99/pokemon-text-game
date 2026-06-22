#!/usr/bin/env python3
"""
Build complete game data from PokeAPI data/v2 CSVs + optional sprites repo.

This is the canonical pipeline — run after cloning:
  git clone --depth 1 https://github.com/PokeAPI/pokeapi.git /tmp/pokeapi
  git clone --depth 1 https://github.com/PokeAPI/sprites.git /tmp/pokeapi-sprites

Usage:
  python3 scripts/build_game_data.py \\
    --pokeapi-root /tmp/pokeapi \\
    --sprites-root /tmp/pokeapi-sprites \\
    --max-species 1025 \\
    --out-dir resources
"""
from __future__ import annotations

import argparse
import csv
import json
import math
import os
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Optional

TYPE_ID_TO_NAME = {
    1: "normal", 2: "fighting", 3: "flying", 4: "poison", 5: "ground",
    6: "rock", 7: "bug", 8: "ghost", 9: "steel", 10: "fire", 11: "water",
    12: "grass", 13: "electric", 14: "psychic", 15: "ice", 16: "dragon",
    17: "dark", 18: "fairy",
}
STAT_ID_TO_KEY = {
    1: "hp", 2: "attack", 3: "defense", 4: "special-attack",
    5: "special-defense", 6: "speed",
}
DAMAGE_CLASS = {1: "status", 2: "physical", 3: "special"}

# ASCII ramp for sprite conversion (dark -> light)
ASCII_RAMP = " .:-=+*#%@"
ASCII_RAMP_BLOCKS = " ░▒▓█"


def read_csv(path: Path) -> list[dict[str, str]]:
    with open(path, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def title_case_poke(name: str) -> str:
    """mr-mime -> Mr-Mime, ho-oh -> Ho-Oh"""
    parts = name.replace("_", "-").split("-")
    return "-".join(p[:1].upper() + p[1:] for p in parts if p)


def build_moves(csv_dir: Path) -> list[dict[str, Any]]:
    names_en: dict[int, str] = {}
    for row in read_csv(csv_dir / "move_names.csv"):
        if row.get("local_language_id") == "9":
            names_en[int(row["move_id"])] = row["name"]

    flavor_en: dict[int, str] = {}
    flavor_path = csv_dir / "move_flavor_text.csv"
    if flavor_path.exists():
        for row in read_csv(flavor_path):
            if row.get("language_id") == "9":
                mid = int(row["move_id"])
                # prefer latest version group; last write wins if unsorted
                txt = row.get("flavor_text", "").replace("\n", " ").replace("\x0c", " ").strip()
                if txt:
                    flavor_en[mid] = txt

    moves = []
    for row in read_csv(csv_dir / "moves.csv"):
        mid = int(row["id"])
        # skip z-moves / max moves etc beyond normal catalog if needed; keep all with names
        if mid not in names_en:
            continue
        tid = int(row["type_id"]) if row.get("type_id") else 1
        dmg_class = int(row.get("damage_class_id") or 1)
        power = int(row["power"]) if row.get("power") else 0
        accuracy = int(row["accuracy"]) if row.get("accuracy") else 0
        pp = int(row["pp"]) if row.get("pp") else 5
        priority = int(row.get("priority") or 0)
        moves.append({
            "id": mid,
            "name": names_en[mid],
            "move_type": TYPE_ID_TO_NAME.get(tid, "normal"),
            "category": DAMAGE_CLASS.get(dmg_class, "status"),
            "power": power,
            "accuracy": accuracy,
            "pp": pp,
            "priority": priority,
            "description": flavor_en.get(mid, ""),
        })
    moves.sort(key=lambda m: m["id"])
    return moves


def build_species(csv_dir: Path, max_species: int) -> list[dict[str, Any]]:
    # English names
    names_en: dict[int, str] = {}
    genus_en: dict[int, str] = {}
    for row in read_csv(csv_dir / "pokemon_species_names.csv"):
        if row.get("local_language_id") == "9":
            sid = int(row["pokemon_species_id"])
            names_en[sid] = row["name"]
            if row.get("genus"):
                genus_en[sid] = row["genus"]

    # Flavor text (pokedex entries) — English, prefer latest
    flavor_en: dict[int, str] = {}
    ft_path = csv_dir / "pokemon_species_flavor_text.csv"
    if ft_path.exists():
        for row in read_csv(ft_path):
            if row.get("language_id") == "9":
                sid = int(row["species_id"])
                txt = row.get("flavor_text", "").replace("\n", " ").replace("\x0c", " ").strip()
                if txt:
                    flavor_en[sid] = txt

    # Capture rate / base happiness from species table
    capture_rate: dict[int, int] = {}
    species_meta: dict[int, dict] = {}
    for row in read_csv(csv_dir / "pokemon_species.csv"):
        sid = int(row["id"])
        capture_rate[sid] = int(row.get("capture_rate") or 45)
        species_meta[sid] = row

    # Default pokemon form stats/types/height/weight (pokemon_id == species_id for mains)
    pstats: dict[int, dict[str, int]] = defaultdict(dict)
    for row in read_csv(csv_dir / "pokemon_stats.csv"):
        pid = int(row["pokemon_id"])
        key = STAT_ID_TO_KEY.get(int(row["stat_id"]))
        if key:
            pstats[pid][key] = int(row["base_stat"])

    ptypes: dict[int, list[tuple[int, str]]] = defaultdict(list)
    for row in read_csv(csv_dir / "pokemon_types.csv"):
        pid = int(row["pokemon_id"])
        tid = int(row["type_id"])
        slot = int(row["slot"])
        ptypes[pid].append((slot, TYPE_ID_TO_NAME.get(tid, "normal")))

    pokemon_rows: dict[int, dict] = {}
    for row in read_csv(csv_dir / "pokemon.csv"):
        pokemon_rows[int(row["id"])] = row

    # Level-up moves for default form (method_id 1 = level-up)
    levelup_moves: dict[int, list[tuple[int, int]]] = defaultdict(list)
    pm_path = csv_dir / "pokemon_moves.csv"
    if pm_path.exists():
        for row in read_csv(pm_path):
            if row.get("pokemon_move_method_id") != "1":
                continue
            if row.get("version_group_id") not in ("20", "18", "16", "15", "14", "10", "7", "6", "5", "3", "2", "1"):
                # accept most; we'll de-dupe by latest level
                pass
            pid = int(row["pokemon_id"])
            mid = int(row["move_id"])
            lvl = int(row.get("level") or 0)
            levelup_moves[pid].append((lvl, mid))

    def default_moves_for(pid: int) -> list[int]:
        entries = levelup_moves.get(pid, [])
        if not entries:
            return [33]  # tackle id in pokeapi is 33
        # take moves learned by level 1-25, prefer highest level among early moves, max 4
        early = sorted([e for e in entries if e[0] <= 25], key=lambda x: x[0])
        if not early:
            early = sorted(entries, key=lambda x: x[0])[:8]
        seen = []
        for _, mid in early:
            if mid not in seen:
                seen.append(mid)
        # also include latest 4 level-up moves overall as filler
        latest = sorted(entries, key=lambda x: x[0], reverse=True)
        for _, mid in latest:
            if mid not in seen:
                seen.append(mid)
            if len(seen) >= 4:
                break
        return seen[:4] if seen else [33]

    species_list = []
    for sid in sorted(names_en.keys()):
        if sid > max_species:
            continue
        # default form usually same id
        pid = sid
        if pid not in pokemon_rows:
            # find first pokemon with this species
            continue
        prow = pokemon_rows[pid]
        types_sorted = [t for _, t in sorted(ptypes.get(pid, [(1, "normal")]))]
        st = pstats.get(pid, {})
        genus = genus_en.get(sid, "")
        flavor = flavor_en.get(sid, "")
        desc = flavor or (f"{genus}." if genus else f"Pokédex entry #{sid}.")
        species_list.append({
            "id": sid,
            "name": names_en[sid],
            "types": types_sorted or ["normal"],
            "base_stats": {
                "hp": st.get("hp", 50),
                "attack": st.get("attack", 50),
                "defense": st.get("defense", 50),
                "sp_attack": st.get("special-attack", 50),
                "sp_defense": st.get("special-defense", 50),
                "speed": st.get("speed", 50),
            },
            "base_experience": int(float(prow.get("base_experience") or 64)),
            "height_dm": int(prow.get("height") or 7),
            "weight_hg": int(prow.get("weight") or 69),
            "description": desc,
            "default_moves": default_moves_for(pid),
            "capture_rate": capture_rate.get(sid, 45),
        })
    return species_list


def png_to_ascii_lines(png_path: Path, width: int = 28, height: int = 14) -> Optional[list[str]]:
    try:
        from PIL import Image
    except ImportError:
        return None
    try:
        im = Image.open(png_path).convert("RGBA")
    except Exception:
        return None

    # Crop transparent margins
    bbox = im.getbbox()
    if bbox:
        im = im.crop(bbox)

    # Resize maintaining aspect into target box
    im = im.resize((width, height), Image.Resampling.LANCZOS)

    lines = []
    for y in range(height):
        row_chars = []
        for x in range(width):
            r, g, b, a = im.getpixel((x, y))
            if a < 32:
                row_chars.append(" ")
                continue
            # luminance
            lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0
            # alpha-weighted: more opaque + brighter -> denser or sparser
            # invert: dark pixels = denser chars (sprites are colorful on transparent)
            idx = int((1.0 - lum) * (len(ASCII_RAMP) - 1))
            if a < 128:
                idx = max(0, idx - 2)
            row_chars.append(ASCII_RAMP[idx])
        lines.append("".join(row_chars).rstrip())
    # trim fully empty top/bottom but pad to at least something
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    if not lines:
        return None
    # normalize width
    w = max(len(l) for l in lines)
    w = max(w, width)
    return [l.ljust(w) for l in lines]


def convert_sprites(sprites_root: Path, out_dir: Path, max_id: int) -> int:
    poke_dir = sprites_root / "sprites" / "pokemon"
    if not poke_dir.is_dir():
        # try direct path
        poke_dir = sprites_root / "pokemon"
    if not poke_dir.is_dir():
        print(f"WARN: sprites dir not found under {sprites_root}", file=sys.stderr)
        return 0
    out_dir.mkdir(parents=True, exist_ok=True)
    count = 0
    for sid in range(1, max_id + 1):
        png = poke_dir / f"{sid}.png"
        if not png.exists():
            continue
        lines = png_to_ascii_lines(png, width=28, height=14)
        if not lines:
            continue
        (out_dir / f"{sid:03d}.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
        count += 1
        if count % 100 == 0:
            print(f"  sprites: {count}…", file=sys.stderr)
    return count


def build_starter_team(species: list[dict]) -> dict:
    """Curated classic team using real names/moves from data."""
    by_id = {s["id"]: s for s in species}
    picks = []
    for sid, hp_override in [(25, 400), (6, 600), (9, 620), (3, 610), (94, 520), (150, 700)]:
        s = by_id.get(sid)
        if not s:
            continue
        picks.append({
            "name": s["name"],
            "moves": [],  # filled by game from species; keep placeholders
            "health": hp_override,
            "type": 0,
        })
    # We'll enrich moves after moves db exists in post step — store move ids as names via caller
    return {"pokemons": picks}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pokeapi-root", type=Path, default=Path("/tmp/pokeapi"))
    ap.add_argument("--sprites-root", type=Path, default=Path("/tmp/pokeapi-sprites"))
    ap.add_argument("--max-species", type=int, default=1025)
    ap.add_argument("--out-dir", type=Path, default=Path("resources"))
    ap.add_argument("--skip-sprites", action="store_true")
    args = ap.parse_args()

    csv_dir = args.pokeapi_root / "data" / "v2" / "csv"
    if not csv_dir.is_dir():
        raise SystemExit(f"PokeAPI CSV dir missing: {csv_dir}\nClone https://github.com/PokeAPI/pokeapi")

    print("Building moves…", file=sys.stderr)
    moves = build_moves(csv_dir)
    print(f"  {len(moves)} moves", file=sys.stderr)

    print("Building species…", file=sys.stderr)
    species = build_species(csv_dir, args.max_species)
    print(f"  {len(species)} species", file=sys.stderr)

    # Resolve default_moves to names for human-readable team file
    move_by_id = {m["id"]: m for m in moves}

    data_dir = args.out_dir / "data"
    data_dir.mkdir(parents=True, exist_ok=True)

    species_db = {
        "species": species,
        "source": f"pokeapi data/v2 csv @ {args.pokeapi_root}",
        "version": "2.1.0",
        "count": len(species),
    }
    (data_dir / "species.json").write_text(json.dumps(species_db, indent=2) + "\n", encoding="utf-8")

    moves_db = {
        "moves": moves,
        "source": f"pokeapi data/v2 csv @ {args.pokeapi_root}",
        "version": "2.1.0",
        "count": len(moves),
    }
    (data_dir / "moves.json").write_text(json.dumps(moves_db, indent=2) + "\n", encoding="utf-8")

    # Team file with real move names from level-up sets
    type_legacy = {
        "electric": 1, "grass": 2, "water": 3, "fire": 4, "psychic": 6,
        "fighting": 7, "ghost": 8, "dragon": 9,
    }
    team_pokes = []
    for sid, hp in [(25, 400), (6, 600), (9, 620), (3, 610), (94, 520), (130, 580), (143, 700), (150, 720)]:
        s = next((x for x in species if x["id"] == sid), None)
        if not s:
            continue
        mnames = []
        for mid in s.get("default_moves", []):
            if mid in move_by_id:
                mnames.append(move_by_id[mid]["name"].replace(" ", ""))
        if not mnames:
            mnames = ["Tackle"]
        prim = s["types"][0] if s["types"] else "normal"
        team_pokes.append({
            "name": s["name"],
            "moves": mnames[:4],
            "health": hp,
            "type": type_legacy.get(prim, 0),
        })
    # Keep the easter egg
    team_pokes.append({
        "name": "Siddharth",
        "moves": ["KillShot", "MegaPunch", "SarcasticComments", "DeadEye"],
        "health": 690,
        "type": 0,
    })
    (args.out_dir / "pokemons.json").write_text(
        json.dumps({"pokemons": team_pokes}, indent=2) + "\n", encoding="utf-8"
    )

    if not args.skip_sprites:
        print("Converting sprites to ASCII…", file=sys.stderr)
        n = convert_sprites(args.sprites_root, args.out_dir / "sprites" / "ascii", args.max_species)
        print(f"  {n} sprites written", file=sys.stderr)
    else:
        print("Skipping sprites", file=sys.stderr)

    print("Done.", file=sys.stderr)
    print(f"  {data_dir / 'species.json'}")
    print(f"  {data_dir / 'moves.json'}")
    print(f"  {args.out_dir / 'pokemons.json'}")


if __name__ == "__main__":
    main()
# After this script, run: python3 scripts/sprites_to_ascii.py
