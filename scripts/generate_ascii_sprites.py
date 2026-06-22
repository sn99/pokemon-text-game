#!/usr/bin/env python3
"""Write a few signature ASCII sprites into resources/sprites/ascii/."""
from pathlib import Path

OUT = Path("resources/sprites/ascii")
OUT.mkdir(parents=True, exist_ok=True)

SPRITES = {
    "001": "      .--.\n     /  oo\\\n    |  (__)\n     \\  ||\n      ||||\n     /_||_\\\n#001",
    "004": "      /^\\\n     / o o\\\n    |  ^  |\n     \\ ~ /\n    /|\\|/|\\\n   ~~ | | ~~\n#004",
    "006": "   ~  /^\\  ~\n    / o o \\\n   <  ===  >\n    \\  V  /\n   ~/|\\|/|\\~\n    /_||_\\\n#006",
    "007": "      .--.\n     (o  o)\n    /| __ |\\\n     | || |\n     /_||_\\\n#007",
    "025": "     (\\__/)\n     (o^.^)\n    z(_(\")(\")\n     /|  |\\\n    * |  | *\n#025",
    "094": "     .-.\n    (o o)\n    | > |\n   /|   |\\\n  <_\\___/_>\n#094",
    "130": "   ~~~~/\\~~~~\n    <(oo)>\n   ~/ || \\~\n    / || \\\n   <__||__>\n#130",
    "143": "    .----.\n   ( o  o )\n   |  __  |\n   |______|\n    /    \\\n#143",
    "150": "      /\\\n     (oo)\n    <(__)>\n     /||\\\n    <_||_>\n#150",
}

for sid, art in SPRITES.items():
    lines = [ln.rstrip() for ln in art.splitlines()]
    while len(lines) < 10:
        lines.append("")
    path = OUT / f"{sid}.txt"
    path.write_text("\n".join(lines[:10]) + "\n")
    print("wrote", path)
