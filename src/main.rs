#![allow(dead_code)]
#![allow(unused_variables)]

use std::io::Write;

mod ast;
mod lexer;

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
  let mut lexer = lexer::Lexer::default();
  lexer.lex(&source_path);
  // lexer.print_tokens();
  // lexer.print_tokens_string();

  let mut parser = ast::Parser::new(&lexer.tokens);
  let ast = parser.parse_program();
  ast.print();

  let code = generate(&ast);
  // println!("{}", code); // String to string literal: let literal = &String[..]

  // Write assembly to file
  let mut file = std::fs::File::create(&assembly_path).unwrap();
  file.write_all(code.as_bytes()).unwrap();
}
