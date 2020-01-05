pub mod data;
pub mod parser;
pub mod scanner;
pub mod interpeter;
pub mod environment;

use clap;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use data::errors;

const INTERPRETER_DIRECTIVE_HELP: &'static str = 
"Interpreter directives:
    .exit - exit the interpreter
    .help - display this text";

fn main() {
    let matches = clap::App::new("TreeLox")
        .version("1.0")
        .author("Reid Swan")
        .about("Tree Walk Lox Interpreter in Rust")
        .arg(
            clap::Arg::with_name("script")
                .help("The script to run")
                .required(false),
        )
        .get_matches();

    if let Some(script) = matches.value_of("script") {
        // script mode
        run_file(script).unwrap();
    } else {
        // REPL mode
        run_prompt().unwrap();
    }
}

fn run_file(script: &str) -> Result<(), errors::LoxError> {
    let file_contents = fs::read_to_string(script).map_err(|e| errors::LoxError::IoError(e))?;
    if let Err(e) = run(&file_contents, &mut interpeter::Interpreter::new(), false) {
        eprintln!("{}", e)
    };
    Ok(())
}

fn run_prompt() -> Result<(), errors::LoxError> {
    let mut input_string = String::new();
    let mut interpeter = interpeter::Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut input_string)
            .map_err(|e| errors::LoxError::IoError(e))?;
        if input_string.trim().starts_with(".") || input_string.trim().is_empty() {
            interpret_directive(&input_string[..]);
        } else if let Err(e) = run(&input_string, &mut interpeter, true) {
            eprintln!("{}", e)
        }
        input_string.clear();
    }
}

fn run(src: &str, interpeter: &mut interpeter::Interpreter, allow_top_level_expr: bool) -> Result<(), String> {
    let mut scanner = scanner::Scanner::new(src);
    let tokens = match scanner.scan_tokens() {
        Err(errors) => {
            let mut err_string = String::from("Failed to scan:\n");
            errors.iter().for_each(|s| {
                err_string = format!("{}\n{:?}", err_string, s)
            });
            return Err(err_string)
        }
        Ok(tokens) => tokens,
    };

    let mut parser = parser::Parser::new(tokens);
    let parse_result = parser.parse();
    let program = match parse_result {
        Ok(program) => program,
        Err(e) => if allow_top_level_expr {
            parser.reset();
            match parser.parse_top_level_expression() {
                Ok(program) => program,
                Err(_) => {
                    return Err(e)
                }
            }
        } else {
            return Err(e)
        }
    };
    interpeter.interpret(program)
}

fn interpret_directive(command: &str) {
    match command.trim() {
        ".exit" => {
            println!("Goodbye");
            exit(0)
        }
        ".help" | "" => {
            println!("{}", INTERPRETER_DIRECTIVE_HELP);
        }
        _ => {
            eprintln!("Unrecognized intepreter directive: {}", command);
            eprintln!("{}", INTERPRETER_DIRECTIVE_HELP);
        }
    }
}
