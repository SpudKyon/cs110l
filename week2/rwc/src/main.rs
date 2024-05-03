use std::{env, io};
use std::fs::File;
use std::io::BufRead;
use std::process;

fn read_file_lines(filename: &String) -> Result<(u32, u32, u32), io::Error> {
    let file = File::open(filename)?;
    let mut num_word: u32 = 0;
    let mut num_line: u32 = 0;
    let mut num_char: u32 = 0;
    for line in io::BufReader::new(file).lines() {
        let line_str = line?;
        if line_str == "" {
            continue;
        }
        let line_word: Vec<&str> = line_str.split("\\s+").collect();
        for word in &line_word {
            num_char += word.len() as u32;
        }
        num_word += line_word.len() as u32;
        num_line += 1;
    }
    Ok((num_word, num_line, num_char))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)
    let (num_word, num_line, num_char) = read_file_lines(filename).expect("can not open file!");
    println!("Num of Word: {}\tNum of line: {}\tNum of character: {}", num_word, num_line, num_char);
}
