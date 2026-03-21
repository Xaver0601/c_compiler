use std::fmt;

pub enum Token {
  OpenBrace,  // {
  CloseBrace, // }
  OpenParen,  // (
  CloseParen, // )
  Semicolon,  // ;
  // Negation,          // -
  // BitwiseComplement, // ~
  // LogicalNegation,   // !
  Keyword(Keyword),
  Expr(Expr),
  Identifier(String),
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Token::OpenBrace => write!(f, "{{"),
      Token::CloseBrace => write!(f, "}}"),
      Token::OpenParen => write!(f, "("),
      Token::CloseParen => write!(f, ")"),
      Token::Semicolon => write!(f, ";"),
      Token::Keyword(Keyword::INT) => write!(f, "KEYWORD_INT"),
      Token::Keyword(Keyword::RETURN) => write!(f, "KEYWORD_RETURN"),
      Token::Identifier(name) => write!(f, "ID({})", name),
      Token::Expr(val) => match val {
        Expr::LiteralInt(n) => write!(f, "INT({})", n),
      },
    }
  }
}

#[derive(Clone, Copy, Default)]
pub enum Keyword {
  INT,
  #[default] // TODO: change this default
  RETURN,
}

pub enum Expr {
  LiteralInt(i32),
}

pub struct Expression {
  pub value: i32,
}

pub struct Statement {
  pub s_type: Keyword,
  pub child_expressions: Vec<Expression>,
}

// #[derive(Default)]
pub struct Function {
  pub name: String,
  pub child_statements: Vec<Statement>,
}

// #[derive(Default)]
pub struct Program {
  pub child_functions: Vec<Function>,
}

impl Expression {
  pub fn print(&self) {
    println!("{}", self.value);
  }
}

impl Statement {
  pub fn print(&self) {
    for expr in &self.child_expressions {
      match self.s_type {
        Keyword::RETURN => print!("    RETURN "),
        Keyword::INT => print!("    INT "),
      }
      expr.print();
    }
  }
}

impl Function {
  pub fn print(&self) {
    for stmt in &self.child_statements {
      println!("FUN {}:\n  body:", self.name);
      stmt.print();
    }
  }
}

impl Program {
  pub fn new() -> Self {
    Program {
      child_functions: vec![],
    }
  }
  pub fn add_function(&mut self, fun: Function) {
    self.child_functions.push(fun);
  }

  pub fn print(&self) {
    for fun in &self.child_functions {
      fun.print();
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
      prog.add_function(self.parse_function());
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

    let expr = self.parse_expression();

    assert!(matches!(self.advance(), Some(Token::Semicolon)));

    Statement {
      s_type,
      child_expressions: vec![expr],
    }
  }

  fn parse_expression(&mut self) -> Expression {
    match self.advance() {
      Some(Token::Expr(Expr::LiteralInt(val))) => Expression { value: *val },
      _ => panic!("Missing int literal"),
    }
  }
}
