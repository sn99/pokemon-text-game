/*
MIT License

Copyright (c) 2019 sn99

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

extern crate serde;
extern crate serde_json;

pub fn input() -> String {
    let mut temp_input = String::new();
    ::std::io::stdin()
        .read_line(&mut temp_input)
        .expect("Error in 'input_message' function !");

    temp_input
}

pub fn i64_input() -> i64 {
    let temp_input = input();
    let _i64: i64 = temp_input.trim().parse().expect("Unable to input a number");

    _i64
}

pub fn players_character_choice(bounds: i64) -> i64 {
    let choice = i64_input();
    if choice > bounds {
        println!(
            "\n========================\nWrong input ... We are out :(\n========================"
        );
        ::std::process::exit(1);
    }

    choice
}
