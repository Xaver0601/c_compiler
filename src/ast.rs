use std::fmt;

pub enum Token {
  OpenBrace,  // {
  CloseBrace, // }
  OpenParen,  // (
  CloseParen, // )
  Semicolon,  // ;
  UnaryOp(UnaryOp),
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
  // BinOp(BinaryOp, Box<Expr>, Box<Expr>),
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Token::OpenBrace => write!(f, "{{"),
      Token::CloseBrace => write!(f, "}}"),
      Token::OpenParen => write!(f, "("),
      Token::CloseParen => write!(f, ")"),
      Token::Semicolon => write!(f, ";"),
      Token::UnaryOp(UnaryOp::Negate) => write!(f, "-"),
      Token::UnaryOp(UnaryOp::BitwiseNot) => write!(f, "~"),
      Token::UnaryOp(UnaryOp::LogicalNot) => write!(f, "!"),
      Token::Keyword(Keyword::INT) => write!(f, "KEYWORD_INT"),
      Token::Keyword(Keyword::RETURN) => write!(f, "KEYWORD_RETURN"),
      Token::Identifier(name) => write!(f, "ID({})", name),
      Token::Expr(val) => match val {
        Expr::LiteralInt(n) => write!(f, "INT({})", n),
        Expr::UnOp(u, expr) => write!(f, "UNOP()<>"),
      },
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
  }
}

impl Expr {
  pub fn print(&self) {
    match self {
      Expr::LiteralInt(val) => {
        println!("{}", val);
      }
      Expr::UnOp(op, operand) => {
        match op {
          UnaryOp::Negate => print!("-"),
          UnaryOp::BitwiseNot => print!("~"),
          UnaryOp::LogicalNot => print!("!"),
        }
        operand.print();
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
    let s_type = match self.advance() {
      Some(Token::Keyword(x)) => *x,
      _ => panic!("Missing keyword"),
    };

    if s_type == Keyword::RETURN {
      let expr = self.parse_expression();
      assert!(matches!(self.advance(), Some(Token::Semicolon)));
      return Statement::Return(expr);
    }
    panic!("Unsupported statement keyword");
  }

  fn parse_expression(&mut self) -> Expr {
    // 1. Try to extract the UnaryOp if the next token is one.
    // We clone/deref the UnaryOp to drop the immutable borrow on `self` immediately.
    let unary_op = match self.peek() {
      Some(Token::UnaryOp(op)) => Some(*op), // assuming `UnaryOp` has #[derive(Clone, Copy)]
      _ => None,
    };

    // 2. If we found one, consume the token and parse the operand
    if let Some(op) = unary_op {
      self.advance(); // consume the UnaryOp token
      let operand = self.parse_expression();
      return Expr::UnOp(op, Box::new(operand)); // Use the specific `op` we found
    }

    match self.advance() {
      Some(Token::Expr(Expr::LiteralInt(val))) => Expr::LiteralInt(*val),
      _ => panic!("Expected expression, found something else"),
    }
  }
}
