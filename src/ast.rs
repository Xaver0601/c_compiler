use std::fmt;

pub enum Token {
  OpenBrace,   // {
  CloseBrace,  // }
  OpenParen,   // (
  CloseParen,  // )
  Semicolon,   // ;
  Minus,       // -
  Tilde,       // ~
  Exclamation, // !
  Plus,        // +
  Star,        // *
  Slash,       // /
  Keyword(Keyword),
  Expr(Expr),
  Identifier(String),
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Keyword {
  INT,
  #[default] // TODO: change this default
  RETURN,
}

#[derive(Clone, Copy)]
pub enum UnaryOp {
  Negate,     // -
  BitwiseNot, // ~
  LogicalNot, // !
}

#[derive(Clone, Copy)]
pub enum BinaryOp {
  Add,      // +
  Subtract, // -
  Multiply, // *
  Divide,   // /
}

pub enum Expr {
  LiteralInt(i32),
  UnOp(UnaryOp, Box<Expr>),
  BinOp(BinaryOp, Box<Expr>, Box<Expr>),
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Token::OpenBrace => write!(f, "{{"),
      Token::CloseBrace => write!(f, "}}"),
      Token::OpenParen => write!(f, "("),
      Token::CloseParen => write!(f, ")"),
      Token::Semicolon => write!(f, ";"),
      Token::Minus => write!(f, "-"),
      Token::Tilde => write!(f, "~"),
      Token::Exclamation => write!(f, "!"),
      Token::Plus => write!(f, "+"),
      Token::Star => write!(f, "*"),
      Token::Slash => write!(f, "/"),
      Token::Keyword(Keyword::INT) => write!(f, "KEYWORD_INT"),
      Token::Keyword(Keyword::RETURN) => write!(f, "KEYWORD_RETURN"),
      Token::Identifier(name) => write!(f, "ID({})", name),
      Token::Expr(val) => match val {
        Expr::LiteralInt(n) => write!(f, "INT({})", n),
        Expr::UnOp(u, expr) => write!(f, "UNOP()<>"),
        Expr::BinOp(u, expr1, expr2) => write!(f, "BINOP()<>"),
      },
      // _ => write!(f, "NOT IMPLEMENTED"),
    }
  }
}

// #[derive(Default)]
pub struct Program {
  pub name: String,
  pub child_functions: Vec<Function>,
}

impl Program {
  pub fn new() -> Self {
    Program {
      name: String::from("unknown"),
      child_functions: vec![],
    }
  }
  pub fn print(&self) {
    println!("{}", self.name);
    for fun in &self.child_functions {
      fun.print();
    }
  }
}

// #[derive(Default)]
pub struct Function {
  pub name: String,
  pub child_statements: Vec<Statement>,
}

impl Function {
  pub fn print(&self) {
    println!("FUN {}:\n  body:", self.name);
    for stmt in &self.child_statements {
      stmt.print();
    }
  }
}

pub enum Statement {
  // DeclareFunction(Keyword, Token),
  Return(Expr),     // return x
  Expression(Expr), // x + 5, !x
}

// impl Statement {
//   pub fn generate(&self) {
//     match self {
//       Statement::Return(x) =>
//     }
//   }
// }

impl Statement {
  pub fn print(&self) {
    match self {
      Statement::Expression(x) => x.print(),
      Statement::Return(x) => {
        print!("    RETURN ");
        x.print()
      }
    }
    println!();
  }
}

impl Expr {
  pub fn print(&self) {
    match self {
      Expr::LiteralInt(val) => {
        print!("{}", val);
      }
      Expr::UnOp(op, operand) => {
        match op {
          UnaryOp::Negate => print!("<UN->"),
          UnaryOp::BitwiseNot => print!("<UN~>"),
          UnaryOp::LogicalNot => print!("<UN!>"),
        }
        operand.print();
      }
      Expr::BinOp(op, operand1, operand2) => {
        operand1.print();
        match op {
          BinaryOp::Add => print!("+"),
          BinaryOp::Subtract => print!("-"),
          BinaryOp::Multiply => print!("*"),
          BinaryOp::Divide => print!("/"),
        }
        operand2.print();
      }
    }
  }
}

pub struct Parser<'a> {
  tokens: &'a Vec<Token>,
  current: usize,
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &'a Vec<Token>) -> Self {
    Parser { tokens, current: 0 }
  }

  // Look at next token without consuming it
  fn peek(&self) -> Option<&Token> {
    self.tokens.get(self.current)
  }

  fn peek_unary_op(&self) -> Option<UnaryOp> {
    match self.peek() {
      Some(Token::Minus) => Some(UnaryOp::Negate),
      Some(Token::Tilde) => Some(UnaryOp::BitwiseNot),
      Some(Token::Exclamation) => Some(UnaryOp::LogicalNot),
      _ => None,
    }
  }

  fn peek_term_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::Star) => Some(BinaryOp::Multiply),
      Some(Token::Slash) => Some(BinaryOp::Divide),
      _ => None,
    }
  }

  fn peek_expr_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::Plus) => Some(BinaryOp::Add),
      Some(Token::Minus) => Some(BinaryOp::Subtract),
      _ => None,
    }
  }

  // Consume current token and move to next one
  fn advance(&mut self) -> Option<&Token> {
    let tok = self.tokens.get(self.current);
    self.current += 1;
    tok
  }

  pub fn parse_program(&mut self) -> Program {
    let mut prog = Program::new();
    while self.peek().is_some() {
      prog.child_functions.push(self.parse_function());
    }
    prog
  }

  fn parse_function(&mut self) -> Function {
    // Expect 'int'
    match self.advance() {
      Some(Token::Keyword(x)) => {}
      _ => panic!("Expected keyword return type"),
    }

    // Expect function name
    let name = match self.advance() {
      Some(Token::Identifier(n)) => n.clone(),
      _ => panic!("Expected function name"),
    };

    // Expect '()'
    assert!(matches!(self.advance(), Some(Token::OpenParen)));
    assert!(matches!(self.advance(), Some(Token::CloseParen)));
    // Expect '{'
    assert!(matches!(self.advance(), Some(Token::OpenBrace)));

    // Parse the inner statements
    let mut statements = Vec::new();
    while !matches!(self.peek(), Some(Token::CloseBrace)) {
      statements.push(self.parse_statement());
      if matches!(statements.last(), Some(Statement::Return(x))) {
        break;
      }
    }

    // Expect '}'
    assert!(matches!(self.advance(), Some(Token::CloseBrace)));

    Function {
      name,
      child_statements: statements,
    }
  }

  fn parse_statement(&mut self) -> Statement {
    let stmt_type = match self.advance() {
      Some(Token::Keyword(x)) => *x,
      _ => panic!("Missing keyword"),
    };

    if stmt_type == Keyword::RETURN {
      let expr = self.parse_expression();
      assert!(matches!(self.advance(), Some(Token::Semicolon)));
      return Statement::Return(expr);
    }
    panic!("Unsupported statement keyword");
  }

  // <exp> ::= <term> { ("+" | "-") <term> }
  fn parse_expression(&mut self) -> Expr {
    let mut left = self.parse_term();
    while let Some(op) = self.peek_expr_op() {
      self.advance(); // consume the + or - token
      let right = self.parse_term();
      left = Expr::BinOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <term> ::= <factor> { ("*" | "/") <factor> }
  fn parse_term(&mut self) -> Expr {
    let mut left = self.parse_factor();
    while let Some(op) = self.peek_term_op() {
      self.advance(); // consume the * or / token
      let right = self.parse_factor();
      left = Expr::BinOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <factor> ::= <unary_op> <factor> | "(" <exp> ")" | <int>
  fn parse_factor(&mut self) -> Expr {
    // 1. Try to extract the UnaryOp if the next token is one.
    // "Check if next token is a UnaryOp, if so, pull the value out of it, name that value op and execute the code in the curly brackets
    // If op is none skip this whole block"
    if let Some(op) = match self.peek() {
      Some(Token::Minus) => Some(UnaryOp::Negate),
      Some(Token::Tilde) => Some(UnaryOp::BitwiseNot),
      Some(Token::Exclamation) => Some(UnaryOp::LogicalNot),
      _ => None,
    } {
      self.advance();
      let operand = self.parse_factor();
      return Expr::UnOp(op, Box::new(operand));
    } else if let Some(op) = match self.peek() {
      // 2. else check if a parenthesis opens
      Some(Token::OpenParen) => Some(Token::OpenParen),
      _ => None,
    } {
      self.advance();
      let node = self.parse_expression(); // Recurse back to top
      if !matches!(self.peek(), Some(Token::CloseParen)) {
        // Check for closing parenthesis
        panic!("Parenthesis not closed!");
      }
      return node;
    } else {
      // 3. Probably just LiteralInt left, but keep match for now
      match self.advance() {
        Some(Token::Expr(Expr::LiteralInt(val))) => return Expr::LiteralInt(*val),
        _ => panic!("Expected expression, found something else"),
      }
    }
  }
}
