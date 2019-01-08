/*MIT License

Copyright (c) 2018 sn99

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

extern crate rand;
extern crate rodio;
extern crate pokemon_text_game;
extern crate serde;

extern crate serde_json;

use rand::Rng;
use pokemon_text_game::extra::*;
use pokemon_text_game::*;
use serde_json::value::Value;
use std::fs::File;
use std::io::{BufReader, Read};

fn edit_character() {
    let mut file = File::open("resources/pokemons.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let json: Value = serde_json::from_str(&contents).unwrap();

    let mut pokemon_vec: Vec<Pokemon> = Vec::new();

    let mut team = read_team_from_file("resources/pokemons.json").expect("Couldn't create team");

    let total_pokemons = team.pokeball.len();

    for i in 0..total_pokemons {
        pokemon_vec.push(team.pokeball[i].clone());
    }

    println!(
        "Pokedex\n\
         ========================\n\
         1.Create new Pokemon\n\
         2.Edit existing Pokemon(only moves and/or health)\n\
         3.Delete a pokemon\n\
         4.Main menu\n\
         ========================"
    );

    let choice = i64_input();

    if choice == 1 {
        let new_pokemon: Pokemon = Pokemon::new();
        team.pokeball.push(new_pokemon);

        write_team_to_file("resources/pokemons.json", &team).expect("could not write to file");
        println!("Press 'q' to exit and any other for main menu ");
        let exit = input();
        if exit.trim() == "q" || exit.trim() == "Q" {
            std::process::exit(1);
        } else {
            game_play();
        }
    } else if choice == 2 {
        println!("\nCharacters Available\n========================");
        for i in 0..total_pokemons {
            println!(
                "{}.{}",
                i + 1,
                &json["pokemons"][i]["name"].as_str().unwrap()
            );
        }

        println!("Enter Character to edit : ");
        let choice = i64_input();
        if choice as usize > total_pokemons {
            println!("\n========================\nWrong input ... \nGoing to main menu\n========================\n");
            game_play();
        }

        println!("The pokemon stats are\n========================");

        team.pokeball[choice as usize - 1].print_details();

        println!("Enter new health : ");
        let new_health = i64_input();
        println!("Enter new moves list : ");

        let temp_new_moves = input();
        let moves: Vec<String> = temp_new_moves
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        team.pokeball[choice as usize - 1].edit(moves, new_health);

        write_team_to_file("resources/pokemons.json", &team).expect("could not write to file");
        println!("Press 'q' to exit and any other for main menu ");
        let exit = input();
        if exit.trim() == "q" || exit.trim() == "Q" {
            std::process::exit(1);
        } else {
            game_play();
        }
    } else if choice == 3 {
        println!("\nCharacters Available\n========================");
        for i in 0..total_pokemons {
            println!(
                "{}.{}",
                i + 1,
                &json["pokemons"][i]["name"].as_str().unwrap()
            );
        }

        println!("Enter character to delete : ");
        let choice = i64_input();

        team.pokeball.remove(choice as usize - 1);

        write_team_to_file("resources/pokemons.json", &team).expect("could not write to file");
        println!("Press 'q' to exit and any other for main menu ");
        let exit = input();
        if exit.trim() == "q" || exit.trim() == "Q" {
            std::process::exit(1);
        } else {
            game_play();
        }
    } else if choice == 4 {
        game_play();
    }
}

fn game_play() {
    let mut file = File::open("resources/pokemons.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let json: Value = serde_json::from_str(&contents).unwrap();

    let mut pokemon_vec: Vec<Pokemon> = Vec::new();

    let team = read_team_from_file("resources/pokemons.json").expect("Couldn't create team");

    let total_pokemons = team.pokeball.len();

    for i in 0..total_pokemons {
        pokemon_vec.push(team.pokeball[i].clone());
    }

    println!("Game loaded !!!\n");

    println!(
        "Menu\n\
         ========================\n\
         1.Play game\n\
         2.Enter Pokedex\n\
         3.Exit\n\
         ========================"
    );

    let choice = i64_input();
    if choice == 1 {
        println!(
            "\n========================\n1.Against Computer\n\
             2.Against Human\n\
             ========================"
        );

        let choice = i64_input();

        println!("\nCharacters Available\n========================");
        for i in 0..total_pokemons {
            println!(
                "{}.{}",
                i + 1,
                &json["pokemons"][i]["name"].as_str().unwrap()
            );
        }
        println!("========================");

        if choice == 1 {
            println!("Challenger choose your Pokemon : ");
            let choice = players_character_choice(total_pokemons as i64);
            let mut player_pokemon = pokemon_vec[choice as usize - 1].clone();

            let mut rng = rand::thread_rng();

            let mut computer_pokemon =
                pokemon_vec[rng.gen_range(1, total_pokemons + 1) - 1].clone();
            println!("\nThe computer chooses {}", computer_pokemon.name);

            let coin_tossed = rng.gen_range(0, 2);

            if coin_tossed == 0 {
                println!("Challenger attacks first !!!\n");
                loop {
                    let dodge_chance = rng.gen_range(0, 8);
                    let player_choice = player_pokemon.choose_attack();

                    if dodge_chance == 0 {
                        println!("{} blocked it !!!", computer_pokemon.name);
                    } else {
                        let critical_chance = rng.gen_range(0, 9);

                        println!(
                            "\nPlayer uses {}",
                            player_pokemon.moves_name[player_choice - 1]
                        );
                        computer_pokemon.damage(critical_chance);

                        if computer_pokemon.health_check() {
                            println!(
                                "\n========================\nPlayer wins with pokemon {}",
                                player_pokemon.name
                            );
                            println!("Press 'q' to exit and any other for main menu ");
                            let exit = input();
                            if exit.trim() == "q" || exit.trim() == "Q" {
                                std::process::exit(1);
                            } else {
                                game_play();
                            }
                        } else {
                            let dodge_chance = rng.gen_range(0, 9);
                            println!("\nComputer's {} attacks now !!!", computer_pokemon.name);
                            if dodge_chance == 0 {
                                println!("Player's {} blocked it !!!", player_pokemon.name);
                            } else {
                                let critical_chance = rng.gen_range(0, 8);
                                println!(
                                    "{} uses {}",
                                    computer_pokemon.name,
                                    computer_pokemon.moves_name
                                        [rng.gen_range(0, computer_pokemon.moves_name.len())]
                                );
                                player_pokemon.damage(critical_chance);

                                if player_pokemon.health_check() {
                                    println!(
                                        "\n========================\nComputer wins with pokemon {}",
                                        computer_pokemon.name
                                    );
                                    println!("Press 'q' to exit and any other for main menu ");
                                    let exit = input();
                                    if exit.trim() == "q" || exit.trim() == "Q" {
                                        std::process::exit(1);
                                    } else {
                                        game_play();
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Computer attacks first !!!");

                loop {
                    let dodge_chance = rng.gen_range(0, 8);

                    if dodge_chance == 0 {
                        println!("Player's {} blocked it !!!", player_pokemon.name);
                    } else {
                        let critical_chance = rng.gen_range(0, 9);
                        println!(
                            "Computer's {} uses {}\n",
                            computer_pokemon.name,
                            computer_pokemon.moves_name
                                [rng.gen_range(0, computer_pokemon.moves_name.len())]
                        );
                        player_pokemon.damage(critical_chance);

                        if player_pokemon.health_check() {
                            println!(
                                "\n========================\nComputer wins with pokemon {}",
                                computer_pokemon.name
                            );
                            println!("Press 'q' to exit and any other for main menu ");
                            let exit = input();
                            if exit.trim() == "q" || exit.trim() == "Q" {
                                std::process::exit(1);
                            } else {
                                game_play();
                            }
                        } else {
                            let dodge_chance = rng.gen_range(0, 9);
                            let player_choice = player_pokemon.choose_attack();

                            if dodge_chance == 0 {
                                println!("{} blocked it !!!", computer_pokemon.name);
                            } else {
                                let critical_chance = rng.gen_range(0, 9);

                                println!(
                                    "Player uses {}",
                                    player_pokemon.moves_name[player_choice - 1]
                                );
                                computer_pokemon.damage(critical_chance);

                                if computer_pokemon.health_check() {
                                    println!(
                                        "\n========================\nPlayer wins with pokemon {}",
                                        player_pokemon.name
                                    );
                                    println!("Press 'q' to exit and any other for main menu ");
                                    let exit = input();
                                    if exit.trim() == "q" || exit.trim() == "Q" {
                                        std::process::exit(1);
                                    } else {
                                        game_play();
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        } else if choice == 2 {
            println!("Player 1 select your pokemon : ");
            let choice = players_character_choice(total_pokemons as i64);
            let mut player1_pokemon = pokemon_vec[choice as usize - 1].clone();
            println!("Player 2 select your pokemon : ");
            let choice = players_character_choice(total_pokemons as i64);
            let mut player2_pokemon = pokemon_vec[choice as usize - 1].clone();

            let mut rng = rand::thread_rng();
            let coin_tossed = rng.gen_range(0, 2);

            if coin_tossed == 0 {
                println!("Player 1 attacks first !!!");

                loop {
                    let dodge_chance = rng.gen_range(0, 8);
                    let player1_choice = player1_pokemon.choose_attack();

                    if dodge_chance == 0 {
                        println!("Player's 2 {} blocked it !!!", player2_pokemon.name);
                    } else {
                        let critical_chance = rng.gen_range(0, 9);

                        println!(
                            "\nPlayer 1 uses {}",
                            player1_pokemon.moves_name[player1_choice - 1]
                        );
                        player2_pokemon.damage(critical_chance);

                        if player2_pokemon.health_check() {
                            println!(
                                "\n========================\nPlayer 1 wins with pokemon {}",
                                player1_pokemon.name
                            );
                            println!("Press and enter any button on the keyboard to exit ");
                            let _exit = input();
                            std::process::exit(1);
                        } else {
                            let dodge_chance = rng.gen_range(0, 9);
                            println!("\nPlayers's 2 {} attacks now !!!", player2_pokemon.name);
                            let player2_choice = player2_pokemon.choose_attack();
                            if dodge_chance == 0 {
                                println!("Player's 1 {} blocked it !!!", player1_pokemon.name);
                            } else {
                                let critical_chance = rng.gen_range(0, 8);
                                println!(
                                    "\nPlayer 2 uses {}",
                                    player2_pokemon.moves_name[player2_choice - 1]
                                );
                                player1_pokemon.damage(critical_chance);

                                if player1_pokemon.health_check() {
                                    println!(
                                        "\n========================\nPlayer 2 wins with pokemon {}",
                                        player2_pokemon.name
                                    );
                                    println!("Press 'q' to exit and any other for main menu ");
                                    let exit = input();
                                    if exit.trim() == "q" || exit.trim() == "Q" {
                                        std::process::exit(1);
                                    } else {
                                        game_play();
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Player 2 attacks first !!!");

                loop {
                    let dodge_chance = rng.gen_range(0, 8);
                    println!("\nPlayers's 2 {} attacks now !!!", player2_pokemon.name);
                    let player2_choice = player2_pokemon.choose_attack();
                    if dodge_chance == 0 {
                        println!("Player's 1 {} blocked it !!!", player1_pokemon.name);
                    } else {
                        let critical_chance = rng.gen_range(0, 9);
                        println!(
                            "\nPlayer 2 uses {}",
                            player2_pokemon.moves_name[player2_choice - 1]
                        );
                        player1_pokemon.damage(critical_chance);

                        if player1_pokemon.health_check() {
                            println!(
                                "\n========================\nPlayer 2 wins with pokemon {}",
                                player2_pokemon.name
                            );
                            println!("Press 'q' to exit and any other for main menu ");
                            let exit = input();
                            if exit.trim() == "q" || exit.trim() == "Q" {
                                std::process::exit(1);
                            } else {
                                game_play();
                            }
                        } else {
                            let dodge_chance = rng.gen_range(0, 9);
                            let player1_choice = player1_pokemon.choose_attack();

                            if dodge_chance == 0 {
                                println!("Player's 2 {} blocked it !!!", player2_pokemon.name);
                            } else {
                                let critical_chance = rng.gen_range(0, 8);

                                println!(
                                    "\nPlayer 1 uses {}",
                                    player1_pokemon.moves_name[player1_choice - 1]
                                );
                                player2_pokemon.damage(critical_chance);

                                if player2_pokemon.health_check() {
                                    println!(
                                        "\n========================\nPlayer 1 wins with pokemon {}",
                                        player1_pokemon.name
                                    );
                                    println!("Press 'q' to exit and any other for main menu ");
                                    let exit = input();
                                    if exit.trim() == "q" || exit.trim() == "Q" {
                                        std::process::exit(1);
                                    } else {
                                        game_play();
                                    }
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!("\n========================\nWrong input ... \nBack to main menu now\n========================\n");
            game_play();
        }
    } else if choice == 2 {
        edit_character();
    } else {
        if choice == 3 {
            println!("Press and enter any button on the keyboard to exit ");
            let _exit = input();
            std::process::exit(1);
        } else {
            println!("\n========================\nWrong input ... \nBack to main menu\n========================\n\n");
            game_play();
        }
    }
}

fn main() {
    println!("Loading game .....");
    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);

    let song_file = File::open("resources/track.mp3").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(song_file)).unwrap());

    game_play();
}
