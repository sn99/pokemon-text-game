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

pub mod extra;

extern crate rand;

extern crate serde;
#[macro_use]
extern crate serde_derive;
// for #[derive(Serialize, Deserialize)]
extern crate serde_json;

use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use crate::extra::*;
use rand::Rng;

pub fn write_team_to_file<P: AsRef<Path>>(
    path: P,
    team: &PokemonsList,
) -> Result<(), Box<dyn Error>> {
    fs::write(path, serde_json::to_string_pretty(team)?)?;
    Ok(())
}

pub fn read_team_from_file<P: AsRef<Path>>(path: P) -> Result<PokemonsList, Box<dyn Error>> {
    let file = File::open(path)?;
    let team = serde_json::from_reader(file)?;
    Ok(team)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonsList {
    #[serde(rename = "pokemons")]
    pub pokeball: Vec<Pokemon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    pub name: String,
    #[serde(rename = "moves")]
    pub moves_name: Vec<String>,
    pub health: i64,
    #[serde(rename = "type")]
    pub pokemon_type: i64,
}

impl Pokemon {
    pub fn edit(&mut self, moves: Vec<String>, health: i64) {
        self.health = health;
        self.moves_name = moves;
    }

    pub fn new() -> Pokemon {
        println!("\n========================\nEnter name of pokemon : ");
        let mut temp_name = input();
        let size_name = temp_name.len();

        temp_name.truncate(size_name - 1);

        println!("Enter moves : ");
        let temp_moves = input();
        let moves: Vec<String> = temp_moves
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        println!("Enter health : ");
        let temp_health = i64_input();

        println!("Enter pokemon type : ");
        let temp_type = i64_input();

        Pokemon {
            name: temp_name.to_owned(),
            moves_name: moves.to_owned(),
            health: temp_health.to_owned(),
            pokemon_type: temp_type.to_owned(),
        }
    }

    pub fn choose_attack(&self) -> usize {
        let mut q = 1;
        for i in &self.moves_name {
            println!("{}.{}", q, i);
            q = q + 1;
        }

        let choice = i64_input();

        choice as usize
    }

    pub fn damage(&mut self, chance: i32) {
        let mut rng = rand::thread_rng();
        let health_lost = rng.gen_range(80, 100);

        random_message();
        if chance == 0 {
            println!("\n{} takes critical damage\n", self.name);
            self.health = self.health - health_lost - rng.gen_range(10, 30);
        } else {
            self.health = self.health - health_lost;
        }
    }

    pub fn health_check(&self) -> bool {
        if self.health <= 0 {
            println!("{} is unable to battle ...", self.name);
            return true;
        } else {
            return false;
        }
    }

    pub fn print_details(&self) {
        println!("{:#?}", self);
    }
}

pub fn random_message() {
    let mut rng = rand::thread_rng();

    match rng.gen_range(0, 4) {
        0 => println!("\nWE can do it"),
        1 => println!("\nNever give up"),
        2 => println!("\nBe the very best"),
        _ => println!("\nTill the end we shall dance"),
    }
}
