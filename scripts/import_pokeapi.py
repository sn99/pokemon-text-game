#!/usr/bin/env python3
"""Import Pokemon species data from PokeAPI v2 CSV dumps or live API.

Preferred source: https://github.com/PokeAPI/pokeapi/tree/master/data/v2

Examples:
  python3 scripts/import_pokeapi.py --live --max-id 151 -o resources/data/species.json
  python3 scripts/import_pokeapi.py --pokeapi-root /path/to/pokeapi -o resources/data/species.json
  python3 scripts/import_pokeapi.py --bootstrap -o resources/data/species.json
"""
from __future__ import annotations
import argparse, csv, json, sys, time, urllib.request
from pathlib import Path
from typing import Any

TYPE_NAMES = {
    1: "normal", 2: "fighting", 3: "flying", 4: "poison", 5: "ground",
    6: "rock", 7: "bug", 8: "ghost", 9: "steel", 10: "fire", 11: "water",
    12: "grass", 13: "electric", 14: "psychic", 15: "ice", 16: "dragon",
    17: "dark", 18: "fairy",
}
STAT_KEYS = {
    1: "hp", 2: "attack", 3: "defense", 4: "special-attack",
    5: "special-defense", 6: "speed",
}

def capitalize_name(name: str) -> str:
    return name[:1].upper() + name[1:] if name else name

def species_record(sid, name, types, stats, base_exp=64, height=7, weight=69, desc=""):
    return {
        "id": sid,
        "name": capitalize_name(name),
        "types": types,
        "base_stats": {
            "hp": stats.get("hp", 50),
            "attack": stats.get("attack", 50),
            "defense": stats.get("defense", 50),
            "sp_attack": stats.get("special-attack", 50),
            "sp_defense": stats.get("special-defense", 50),
            "speed": stats.get("speed", 50),
        },
        "base_experience": base_exp,
        "height_dm": height,
        "weight_hg": weight,
        "description": desc or f"Data imported from PokeAPI (#{sid}).",
        "default_moves": [1],
        "capture_rate": 45,
    }

def import_live(max_id: int):
    out = []
    for i in range(1, max_id + 1):
        url = f"https://pokeapi.co/api/v2/pokemon/{i}"
        print(f"GET {url}", file=sys.stderr)
        try:
            with urllib.request.urlopen(url, timeout=30) as resp:
                data = json.load(resp)
        except Exception as e:
            print(f"  skip {i}: {e}", file=sys.stderr)
            continue
        types = [t["type"]["name"] for t in sorted(data["types"], key=lambda x: x["slot"])]
        stats = {s["stat"]["name"]: s["base_stat"] for s in data["stats"]}
        out.append(species_record(
            data["id"], data["name"], types, stats,
            base_exp=data.get("base_experience") or 64,
            height=data.get("height") or 7,
            weight=data.get("weight") or 69,
        ))
        time.sleep(0.05)
    return out

def import_from_pokeapi_csvs(root: Path):
    csv_dir = root / "data" / "v2" / "csv"
    if not csv_dir.is_dir():
        raise SystemExit(f"CSV dir not found: {csv_dir}")
    names = {}
    with open(csv_dir / "pokemon_species_names.csv", newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            if row.get("local_language_id") == "9":
                names[int(row["pokemon_species_id"])] = row["name"]
    ptypes = {}
    with open(csv_dir / "pokemon_types.csv", newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            pid = int(row["pokemon_id"])
            tid = int(row["type_id"])
            slot = int(row["slot"])
            ptypes.setdefault(pid, []).append((slot, TYPE_NAMES.get(tid, "normal")))
    pstats = {}
    with open(csv_dir / "pokemon_stats.csv", newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            pid = int(row["pokemon_id"])
            key = STAT_KEYS.get(int(row["stat_id"]))
            if key:
                pstats.setdefault(pid, {})[key] = int(row["base_stat"])
    heights, weights, base_exp = {}, {}, {}
    with open(csv_dir / "pokemon.csv", newline="", encoding="utf-8") as f:
        for row in csv.DictReader(f):
            pid = int(row["id"])
            heights[pid] = int(row.get("height") or 7)
            weights[pid] = int(row.get("weight") or 69)
            base_exp[pid] = int(float(row.get("base_experience") or 64))
    out = []
    for sid in sorted(names.keys()):
        types_sorted = [t for _, t in sorted(ptypes.get(sid, [(1, "normal")]))]
        out.append(species_record(
            sid, names[sid], types_sorted or ["normal"], pstats.get(sid, {}),
            base_exp=base_exp.get(sid, 64), height=heights.get(sid, 7), weight=weights.get(sid, 69),
        ))
    return out

def bootstrap_minimal():
    rows = [
        (1, "Bulbasaur", ["grass", "poison"], 45, 49, 49, 65, 65, 45, 64),
        (4, "Charmander", ["fire"], 39, 52, 43, 60, 50, 65, 62),
        (6, "Charizard", ["fire", "flying"], 78, 84, 78, 109, 85, 100, 240),
        (7, "Squirtle", ["water"], 44, 48, 65, 50, 64, 43, 63),
        (25, "Pikachu", ["electric"], 35, 55, 40, 50, 50, 90, 112),
        (150, "Mewtwo", ["psychic"], 106, 110, 90, 154, 90, 130, 306),
    ]
    out = []
    for sid, name, types, hp, atk, df, spa, spd, spe, bx in rows:
        out.append(species_record(sid, name, types, {
            "hp": hp, "attack": atk, "defense": df,
            "special-attack": spa, "special-defense": spd, "speed": spe,
        }, base_exp=bx, desc=f"{name} — bootstrap entry."))
    return out

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("-o", "--output", default="resources/data/species.json")
    ap.add_argument("--live", action="store_true")
    ap.add_argument("--max-id", type=int, default=151)
    ap.add_argument("--pokeapi-root", type=Path)
    ap.add_argument("--bootstrap", action="store_true")
    args = ap.parse_args()
    if args.live:
        species, source = import_live(args.max_id), "pokeapi.co live"
    elif args.pokeapi_root:
        species, source = import_from_pokeapi_csvs(args.pokeapi_root), f"pokeapi csv @ {args.pokeapi_root}"
    else:
        species, source = bootstrap_minimal(), "bootstrap"
    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps({"species": species, "source": source, "version": "2.0.0"}, indent=2) + "\n")
    print(f"Wrote {len(species)} species -> {out_path}")

if __name__ == "__main__":
    main()
