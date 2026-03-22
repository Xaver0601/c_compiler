#![allow(dead_code)]
#![allow(unused_variables)]

use std::io::Write;
use std::process::Command;
mod ast;
mod generator;
mod lexer;
mod parser;

fn main() {
  let source_path = String::from("temp/temp.c");
  let assembly_path = String::from("temp/temp.s");

  // Read tokens
  let mut lexer = lexer::Lexer::default();
  lexer.lex(&source_path);
  // lexer.print_tokens();
  // lexer.print_tokens_string();

  let mut parser = parser::Parser::new(&lexer.tokens);
  let mut program = parser.parse_program();
  program.print();

  let generator = generator::Generator { ast: program };
  let code = generator.generate_program();
  // println!("{}", code); // String to string literal: let literal = &String[..]

  // Write assembly to file
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
