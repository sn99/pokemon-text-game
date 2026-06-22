#!/usr/bin/env python3
"""Convert PokeAPI sprites PNGs to ASCII using chafa block symbols.

Requires: chafa (https://hpjansson.org/chafa/), and a clone of
https://github.com/PokeAPI/sprites with sprites/pokemon/*.png
"""
from __future__ import annotations
import argparse, re, subprocess, sys
from pathlib import Path

ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\].*?(?:\x07|\x1b\\)")

def chafa_lines(png: Path, cols: int = 34, rows: int = 16) -> list[str] | None:
    # Do NOT pass --colors none — it fills the canvas with ▔ and destroys silhouettes.
    cmd = [
        "chafa", str(png),
        f"--size={cols}x{rows}",
        "-f", "symbols",
        "--symbols", "block",
        "--stretch",
        "--optimize", "0",
    ]
    r = subprocess.run(cmd, capture_output=True)
    if r.returncode != 0:
        return None
    text = r.stdout.decode("utf-8", errors="replace")
    text = ANSI_RE.sub("", text)
    lines = [ln.rstrip() for ln in text.splitlines()]

    def nonempty(s: str) -> bool:
        t = s.strip()
        if not t:
            return False
        return any(c not in " ░▔▁▂ " for c in t)

    while lines and not nonempty(lines[0]):
        lines.pop(0)
    while lines and not nonempty(lines[-1]):
        lines.pop()
    if not lines:
        return None
    min_lead = min((len(l) - len(l.lstrip())) for l in lines if l.strip())
    lines = [l[min_lead:].rstrip() for l in lines]
    w = max(len(l) for l in lines)
    return [l.ljust(w) for l in lines]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sprites-root", type=Path, default=Path("/tmp/pokeapi-sprites"))
    ap.add_argument("--out-dir", type=Path, default=Path("resources/sprites/ascii"))
    ap.add_argument("--max-id", type=int, default=1025)
    args = ap.parse_args()
    poke = args.sprites_root / "sprites" / "pokemon"
    if not poke.is_dir():
        raise SystemExit(f"Missing {poke}; clone https://github.com/PokeAPI/sprites")
    args.out_dir.mkdir(parents=True, exist_ok=True)
    n = 0
    for sid in range(1, args.max_id + 1):
        png = poke / f"{sid}.png"
        if not png.exists():
            continue
        lines = chafa_lines(png)
        if not lines:
            continue
        (args.out_dir / f"{sid:03d}.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
        n += 1
        if n % 150 == 0:
            print(f"{n}…", file=sys.stderr)
    print(f"Wrote {n} sprites -> {args.out_dir}", file=sys.stderr)

if __name__ == "__main__":
    main()
