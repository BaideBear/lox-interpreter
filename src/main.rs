use lox_interpreter::{lexer::Lexer, parser::Parser, Expr, Literal, Stmt};
use std::{
    collections::HashMap, fs, io::{self, Write}, path::Path
};
use lox_interpreter::token::Token;
mod intepreter;

fn main() {
    println!("Lox Interpreter (Rust)");
    println!("Usage: ");
    println!("  Interactive mode: run without arguments");
    println!("  File mode: provide a .lox file path\n");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        process_file(file_path);
    } else {
        interactive_mode();
    }
}

fn interactive_mode() {
    println!("Entering interactive mode...");
    println!("Type Lox expressions or 'exit' to quit\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() == "exit" {
            break;
        }

        process_input(&input);
    }
}

fn process_file(file_path: &str) {
    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("Error: File not found - {}", file_path);
        return;
    }

    if path.extension().map_or(true, |ext| ext != "lox") {
        eprintln!("Error: Expected .lox file");
        return;
    }

    match fs::read_to_string(path) {
        Ok(source) => {
            println!("Parsing file: {}\n", file_path);
            process_input(&source);
        }
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}

fn process_input(input: &str) {
    if input.trim().is_empty() {
        return;
    }

    // 词法分析
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token();
        if token == Token::Eof {
            break;
        }
        tokens.push(token);
    }

    println!("\nTokens:");
    for token in &tokens {
        println!("  {:?}", token);
    }

    // 语法分析
    let mut parser = Parser::new(&tokens);
    println!("\nAST:");
    let statements = parser.parse();  // 直接获取Vec<Stmt>
    
    /*for stmt in statements {
        println!("{:#?}", stmt);
    }*/
    let mut map: HashMap<String, Literal> = HashMap::new();
    let mut func: HashMap<String,(Vec<Token>, Vec<Stmt>)> = HashMap::new();
    intepreter::traverse_statements(&statements,0,&mut map,&mut func);
}
