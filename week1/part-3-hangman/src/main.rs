// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;

use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::process::id;

const NUM_INCORRECT_GUESSES: usize = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn display(v: &Vec<char>) {
    for x in v {
        print!("{}", x)
    }
    println!()
}

fn guess_a_char(g_c: &mut Vec<char>, swc: &mut Vec<char>, g: &mut Vec<char>, idx: usize, n: usize) -> usize {
    print!("The word so far is: ");
    display(&g_c);
    print!("You hava guessed the following letters: ");
    display(&g);
    println!("You have {} guesses left", NUM_INCORRECT_GUESSES - idx);
    print!("Please guess a letter: ");
    // Make sure the prompt from the previous line gets displayed:
    io::stdout()
        .flush()
        .expect("Error flushing stdout.");
    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("Error reading line.");
    let c: Vec<char> = guess.chars().collect();
    let mut flag: bool = false;
    g.push(c[0]);
    for i in 0..n {
        if swc[i] == c[0] {
            g_c[i] = c[0];
            swc[i] = ' ';
            flag = true;
            break;
        }
    }
    if !flag { // wrong answer
        println!("Sorry that letter is not in the word.");
        1
    } else {
        0
    }
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let mut secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    let n = secret_word_chars.len();
    let mut guessed_corrected: Vec<char> = "-".repeat(n).chars().collect();
    let mut gussed: Vec<char> = Vec::new();
    let mut counter: usize = 0;
    let mut cnt: usize = 0;
    println!("Welcome to CS110L Hangman!");
    loop {
        if cnt - counter == n {
            println!("Congratulations you guessed the secret word: {}", secret_word);
            break;
        }
        if counter >= NUM_INCORRECT_GUESSES {
            println!("Sorry, you ran out of guesses!");
            break;
        }
        counter += guess_a_char(&mut guessed_corrected, &mut secret_word_chars, &mut gussed, counter, n);
        cnt += 1;
        println!();
    }
}
