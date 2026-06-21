# CLI based basic pokemon game
[![Build Status](https://travis-ci.com/sn99/pokemon-text-game.svg?branch=master)](https://travis-ci.com/sn99/pokemon-text-game)
[![Build status](https://ci.appveyor.com/api/projects/status/94r186vjkirtxjak?svg=true)](https://ci.appveyor.com/project/sn99/pokemon-text-game)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=sn99/pokemon-text-game)](https://dependabot.com)

## Prerequisites (all platforms)

Install [Rust](https://rustup.rs/) (`rustc` and `cargo`) before building. Run commands from the project root so `resources/pokemons.json` and `resources/track.mp3` are found.

Background music uses [rodio](https://crates.io/crates/rodio) and needs ALSA development headers on Linux to **build** (see distro sections below).

## For Linux (Debian / Ubuntu / Mint, etc.)

1. Clone the [repository](https://github.com/sn99/pokemon-text-game/archive/master.zip) and extract it, or `git clone` the repo.
2. Install build dependencies for audio:
   ```bash
   sudo apt update
   sudo apt install pkg-config libasound2-dev
   ```
3. Build and run:
   ```bash
   cargo build
   cargo run
   cargo test
   ```

## For Linux (Fedora / RHEL / CentOS Stream / Rocky / AlmaLinux)

1. Clone the [repository](https://github.com/sn99/pokemon-text-game/archive/master.zip) and extract it, or `git clone` the repo.
2. If you do not have Rust yet, install it via [rustup](https://rustup.rs/) (recommended), or on Fedora you can also use:
   ```bash
   sudo dnf install rust cargo
   ```
3. Install build dependencies for audio:
   ```bash
   sudo dnf install pkgconf-pkg-config alsa-lib-devel
   ```
   On older RHEL/CentOS with `yum`:
   ```bash
   sudo yum install pkgconfig alsa-lib-devel
   ```
4. Build and run:
   ```bash
   cargo build
   cargo run
   cargo test
   ```

## Controls (TUI)

Works the same on all Linux distros and terminals that support a normal TTY:

- **↑/↓** or **j/k** — navigate menus and lists
- **Enter** / **Space** — select / confirm / attack
- **Tab** / **Shift+Tab** — switch form fields (create/edit Pokemon)
- **q** / **Esc** — go back (or quit from main menu)
- **Ctrl+C** — quit immediately

## For Windows

1. Clone the [repository](https://github.com/sn99/pokemon-text-game/archive/master.zip) (extract the contents) or just download the [resources](https://github.com/sn99/pokemon-text-game/tree/master/resources) file as it is (a file name resources that has both track.mp3 and pokemons.json)
2. [Download](https://github.com/sn99/pokemon-text-game/releases/download/v1.0.0/pokemon-text-game.exe) the executable
3. Put "resources" folder and "pokemon-text-game.exe" in the same folder and run the ".exe" file to play the game :)

## Building on Linux for Windows

Cross-compile a Windows `.exe` from Linux (package names differ by distro).

**Debian / Ubuntu:**
```bash
sudo apt install pkg-config libasound2-dev gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target=x86_64-pc-windows-gnu --verbose
```

**Fedora / RHEL family:**
```bash
sudo dnf install pkgconf-pkg-config alsa-lib-devel mingw64-gcc
rustup target add x86_64-pc-windows-gnu
cargo build --release --target=x86_64-pc-windows-gnu --verbose
```

***rustc and cargo should be installed for building the game***
