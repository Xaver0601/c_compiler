#![allow(dead_code)]
#![allow(unused_variables)]

use regex::Regex;
use std::fs;
use std::io::Write;

mod ast;

fn lex_string(path: &String) -> Vec<String> {
  let content = fs::read_to_string(path).expect("Could not read file");
  // println!("{content}");
  let re =
    Regex::new(r"(\{)|(\})|(\()|(\))|(\;)|(int)\b|(return)\b|([a-zA-Z]\w*)|([0-9]+)").unwrap();
  let hay = &content;

  let mut results: Vec<String> = Vec::new();
  for cap in re.captures_iter(hay) {
    results.push(String::from(&cap[0]));
  }
  results
}

fn lex_token(path: &String) -> Vec<ast::Token> {
  let content = fs::read_to_string(path).expect("Could not read file");
  // println!("{content}");
  let re =
    Regex::new(r"(\{)|(\})|(\()|(\))|(\;)|(int)\b|(return)\b|([a-zA-Z]\w*)|([0-9]+)").unwrap();
  let mut results: Vec<ast::Token> = Vec::new();
  for cap in re.captures_iter(&content) {
    let token = if cap.get(1).is_some() {
      ast::Token::OpenBrace
    } else if cap.get(2).is_some() {
      ast::Token::CloseBrace
    } else if cap.get(3).is_some() {
      ast::Token::OpenParen
    } else if cap.get(4).is_some() {
      ast::Token::CloseParen
    } else if cap.get(5).is_some() {
      ast::Token::Semicolon
    } else if cap.get(6).is_some() {
      ast::Token::Keyword(ast::Keyword::INT)
    } else if cap.get(7).is_some() {
      ast::Token::Keyword(ast::Keyword::RETURN)
    } else if let Some(m) = cap.get(8) {
      ast::Token::Identifier(m.as_str().to_string())
    } else if let Some(m) = cap.get(9) {
      let val = m.as_str().parse().expect("Not a number");
      ast::Token::Expr(ast::Expr::LiteralInt(val))
    } else {
      continue;
    };

    results.push(token);
  }
  results
}

fn parse(tokens: &Vec<ast::Token>) -> ast::Program {
  let mut parser = ast::Parser::new(tokens);
  parser.parse_program()
}

fn generate(ast: &ast::Program) -> String {
  let mut assembly = String::new();
  for fun in &ast.child_functions {
    assembly += &format!(".globl {name}\n{name}:\n", name = fun.name);
    assembly += &format!("{}", generate_statement(&fun));
  }
  assembly += "ret\n.section .note.GNU-stack,\"\",@progbits\n";
  assembly
}

fn generate_statement(fun: &ast::Function) -> String {
  let mut stmt = String::new();
  for stm in &fun.child_statements {
    match stm.s_type {
      ast::Keyword::RETURN => stmt += &format!("movl ${}, %eax\n", stm.child_expressions[0].value),
      _ => {}
    }
  }
  stmt
}

fn main() {
  let source_path = String::from("temp/temp.c");
  let assembly_path = String::from("temp/temp.s");

  // Read capture for capture
  // let x = lex_string(&source_path);
  // for capture in x {
  //     println!("{}", capture);
  // }

  // Read tokens
  let tokens = lex_token(&source_path);
  // for tok in &tokens {
  //   println!("{}", tok);
  // }

  let ast = parse(&tokens);
  // ast.print();

  let code = generate(&ast);
  println!("{}", code); // String to string literal: let literal = &String[..]

  // Write assembly to file
  let mut file = std::fs::File::create(&assembly_path).unwrap();
  file.write_all(code.as_bytes()).unwrap();
}
