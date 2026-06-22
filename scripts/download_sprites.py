#!/usr/bin/env python3
"""Download Gen 1 front/back sprites from PokeAPI/sprites into assets/."""

import concurrent.futures
import os
import urllib.request

BASE = "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon"
ROOT = os.path.join(os.path.dirname(__file__), "..", "assets", "sprites", "pokemon")


def dl(args):
    i, sub = args
    url = f"{BASE}{sub}/{i}.png"
    out_dir = os.path.join(ROOT, sub.lstrip("/")) if sub else ROOT
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, f"{i}.png")
    if os.path.exists(path) and os.path.getsize(path) > 50:
        return True
    try:
        urllib.request.urlretrieve(url, path)
        return True
    except Exception as e:
        print(f"fail {sub}/{i}: {e}")
        return False


def main():
    tasks = [(i, s) for i in range(1, 152) for s in ("", "/back")]
    ok = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=16) as ex:
        for r in ex.map(dl, tasks):
            if r:
                ok += 1
    print(f"ok {ok}/{len(tasks)} -> {os.path.abspath(ROOT)}")


if __name__ == "__main__":
    main()