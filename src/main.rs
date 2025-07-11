use std::env;
use std::fs;
mod parser;
mod tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  {} tokenize <file.lox>", args[0]);
        eprintln!("  {} evaluate <file.lox>", args[0]);
        eprintln!("  {} parse <file.lox>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file: {}", filename);
        String::new()
    });

    match command.as_str() {
        "tokenize" => tokenizer::run_tokenizer(&file_contents),
        "parse" => parser::run_parser(&file_contents),
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
