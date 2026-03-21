#![allow(dead_code)]
#![allow(unused_variables)]

use regex::Regex;
use std::fs;
use std::io::Write;

mod ast;

fn lex(path: &String) -> (Vec<ast::Token>, Vec<String>) {
  let content = fs::read_to_string(path).expect("Could not read file");
  // println!("{content}");
  let re = Regex::new(
    r"(\{)|(\})|(\()|(\))|(\;)|(int)\b|(return)\b|([a-zA-Z]\w*)|([0-9]+)|(\-)|(\~)|(\!)|(\+)|(\*)|(\/)",
  )
  .unwrap();
  let mut tokens: Vec<ast::Token> = Vec::new();
  let mut token_strings: Vec<String> = Vec::new();
  for cap in re.captures_iter(&content) {
    token_strings.push(String::from(&cap[0]));
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
    } else if cap.get(10).is_some() {
      ast::Token::Minus
    } else if cap.get(11).is_some() {
      ast::Token::Tilde
    } else if cap.get(12).is_some() {
      ast::Token::Exclamation
    } else if cap.get(13).is_some() {
      ast::Token::Plus
    } else if cap.get(14).is_some() {
      ast::Token::Star
    } else if cap.get(15).is_some() {
      ast::Token::Slash
    } else {
      continue;
    };
    tokens.push(token);
  }
  (tokens, token_strings)
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
    match stm {
      ast::Statement::Return(x) => {
        stmt += &generate_expression(x);
        // stmt += "movl %eax, %eax\n";
      }
      ast::Statement::Expression(x) => {
        stmt += &generate_expression(x);
      }
    }
  }
  stmt
}

fn generate_expression(expr: &ast::Expr) -> String {
  match expr {
    ast::Expr::LiteralInt(val) => format!("movl ${}, %eax\n", val),
    ast::Expr::UnOp(op, operand) => {
      let mut asm = String::new();
      asm += &generate_expression(operand);
      match op {
        ast::UnaryOp::Negate => {
          asm += "negl %eax\n";
        }
        ast::UnaryOp::BitwiseNot => {
          asm += "notl %eax\n";
        }
        ast::UnaryOp::LogicalNot => {
          asm += "cmpl $0, %eax\n"; // if (%eax - 0) == 0 this sets the ZF flag to 1
          asm += "movl $0, %eax\n"; // zero out %eax
          asm += "sete %al\n"; // sete sets value to 1 if ZF flag is 1; can only modify 1 byte, so we use %al (lower byte of %eax)
        }
      }
      asm
    }
    ast::Expr::BinOp(op, operand1, operand2) => {
      let mut asm = String::new();
      asm += &generate_expression(operand1);
      match op {
        ast::BinaryOp::Add => {}
        ast::BinaryOp::Subtract => {}
        ast::BinaryOp::Multiply => {}
        ast::BinaryOp::Divide => {}
      }
      asm
    }
  }
}

fn main() {
  let source_path = String::from("temp/temp.c");
  let assembly_path = String::from("temp/temp.s");

  // Read tokens
  let (tokens, token_strings) = lex(&source_path);
  for tok in &tokens {
    print!("{} ", tok);
  }
  // for tok_str in &token_strings {
  //   println!("{}", tok_str);
  // }

  let ast = parse(&tokens);
  ast.print();

  let code = generate(&ast);
  // println!("{}", code); // String to string literal: let literal = &String[..]

  // Write assembly to file
  let mut file = std::fs::File::create(&assembly_path).unwrap();
  file.write_all(code.as_bytes()).unwrap();
}
