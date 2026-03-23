#![allow(dead_code)]
#![allow(unused_variables)]

use std::fs;
use std::io::Write;
use std::process::Command;

use compiler::generator;
use compiler::lexer;
use compiler::parser;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    panic!("Please provide at least one argument.");
  }
  // let source_path = String::from("temp/temp.c");
  let source_path = String::from(&args[1]);
  let content = fs::read_to_string(&source_path).expect("Could not read file");

  // Read tokens
  let mut lexer = lexer::Lexer::default();
  lexer.lex(content);
  // lexer.print_tokens();
  // lexer.print_tokens_literal();

  let mut parser = parser::Parser::new(lexer.tokens); // Take ownership of tokens
  let mut program = parser.parse_program(&source_path);
  program.build_pretty_ast();
  // program.print();

  let generator = generator::Generator { ast: program };
  let code = generator.generate_program();
  // println!("{}", code); // String to string literal: let literal = &String[..]

  // Write assembly to file
  let assembly_path = String::from("temp/temp.s");
  let mut file = std::fs::File::create(&assembly_path).unwrap();
  file.write_all(code.as_bytes()).unwrap();

  let status = Command::new("gcc")
    .arg(assembly_path) // Add a single argument
    .args(["-o", "temp/temp.out"]) // Add multiple arguments at once
    .status() // Execute and wait for finish
    .expect("Failed to execute gcc");
  if !status.success() {
    panic!("Compilation failed with status: {}", status);
  }

  let status = Command::new("./temp/temp.out")
    .status()
    .expect("Failed to execute binary");
  println!("Execution exit with status: {}", status);
}
