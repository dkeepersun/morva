use std::env;
use std::fs;
use std::process::ExitCode;

use morva_core::{Declaration, check, parse};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command] if command == "help" || command == "--help" || command == "-h" => {
            help();
            ExitCode::SUCCESS
        }
        [command, path] if command == "check" || command == "parse" => run(command, path),
        _ => {
            help();
            ExitCode::from(2)
        }
    }
}

fn run(command: &str, path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: cannot read {path}: {error}");
            return ExitCode::from(2);
        }
    };
    let document = match parse(&source) {
        Ok(document) => document,
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("error: {diagnostic}");
            }
            return ExitCode::FAILURE;
        }
    };
    let diagnostics = check(&document);
    if !diagnostics.is_empty() {
        for diagnostic in diagnostics {
            eprintln!("error: {diagnostic}");
        }
        return ExitCode::FAILURE;
    }
    if command == "parse" {
        for declaration in &document.declarations {
            print_declaration(declaration, 0);
        }
    } else {
        println!("ok: {path}");
    }
    ExitCode::SUCCESS
}

fn print_declaration(declaration: &Declaration, depth: usize) {
    println!(
        "{}{} {}",
        "  ".repeat(depth),
        declaration.kind,
        declaration.name
    );
    for child in &declaration.declarations {
        print_declaration(child, depth + 1);
    }
}

fn help() {
    println!(
        "Morva semantic model tools\n\nUsage:\n  morva check <file>\n  morva parse <file>\n  morva help"
    );
}
