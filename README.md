# CLI based basic pokemon game
[![Build Status](https://travis-ci.com/sn99/pokemon-text-game.svg?branch=master)](https://travis-ci.com/sn99/pokemon-text-game)
[![Build status](https://ci.appveyor.com/api/projects/status/94r186vjkirtxjak?svg=true)](https://ci.appveyor.com/project/sn99/pokemon-text-game)
## For linux
1. Clone the [repository](https://github.com/sn99/pokemon-text-game/archive/master.zip) and extract the contents in the same file
2. Install the following packages
    1. `sudo apt install pkg-config`
    2. `sudo apt install libasound2-dev`
3. Use cargo build in the same dictionary to build the program and use cargo run to play the game

## For Windows
1. Clone the [repository](https://github.com/sn99/pokemon-text-game/archive/master.zip) (extract the contents) or just download the [resources](https://github.com/sn99/rust_sample_game/tree/master/resources) file as it is (a file name resources that has both track.mp3 and pokemons.json)
2. [Download](https://github.com/sn99/pokemon-text-game/releases/download/v0.3/pokemon-text-game.exe) the executable
3. Put "resources" folder and "pokemon-text-game.exe" in the same folder and run the ".exe" file to play the game :)

## Building on linux for windows
1. `sudo apt install pkg-config`
2. `sudo apt install libasound2-dev`
3. `sudo apt-get install gcc-mingw-w64-x86-64 -y`
4. `rustup target add x86_64-pc-windows-gnu`
5. `cargo build --release --target=x86_64-pc-windows-gnu --verbose`

***rustc and cargo should be installed for building the game***
