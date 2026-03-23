use crate::ast::*;
use crate::lexer::{Keyword, Token};

// Converts raw tokens into an AST
pub struct Parser {
  tokens: Vec<Token>,
  current: usize,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Parser { tokens, current: 0 }
  }

  // Look at next token without consuming it
  fn peek(&self) -> Option<&Token> {
    self.tokens.get(self.current)
  }

  // Check if next token is a unary operator (-, ~, !)
  fn peek_unary_op(&self) -> Option<UnaryOp> {
    match self.peek() {
      Some(Token::Minus) => Some(UnaryOp::Negate),
      Some(Token::Tilde) => Some(UnaryOp::BitwiseNot),
      Some(Token::Exclamation) => Some(UnaryOp::LogicalNot),
      _ => None,
    }
  }

  // Check if next token is a term operator (*, /)
  fn peek_term_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::Star) => Some(BinaryOp::Multiply),
      Some(Token::Slash) => Some(BinaryOp::Divide),
      _ => None,
    }
  }

  // Check if next token is a expression operator (+, -)
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

  // Initialize recursive parsing of program
  // <program> ::= <function>
  pub fn parse_program(&mut self, path: &String) -> Program {
    let mut prog = Program::new(String::from(path));
    while self.peek().is_some() {
      prog.child_functions.push(self.parse_function());
    }
    prog
  }

  // <function> ::= "int" <id> "(" ")" "{" <statement> "}"
  fn parse_function(&mut self) -> Function {
    // Function has to start with 'int'
    match self.advance() {
      Some(Token::Keyword(Keyword::INT)) => {}
      _ => panic!("Expected INT keyword"),
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
      // If the last statement was a 'return' the function ends
      if matches!(statements.last(), Some(Statement::Return(_x))) {
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

  // <statement> ::= "return" <exp> ";"
  fn parse_statement(&mut self) -> Statement {
    match self.advance() {
      Some(Token::Keyword(Keyword::RETURN)) => {
        let expr = self.parse_expression();
        match self.advance() {
          Some(Token::Semicolon) => {} // All good, do nothing
          Some(other) => panic!("Expected ';' after statement, found: {}", other),
          None => panic!("Expected ';' after statement, found EOF (End of File)"),
        }
        return Statement::Return(expr);
      }
      Some(other_token) => panic!("Unsupported statement starting with token: {}", other_token),
      _ => panic!("Unsupported keyword"),
    };
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
    if let Some(op) = self.peek_unary_op() {
      self.advance();
      let operand = self.parse_factor();
      return Expr::UnOp(op, Box::new(operand));
    } else if let Some(_op) = match self.peek() {
      // 2. else check if a parenthesis opens
      Some(Token::OpenParen) => Some(Token::OpenParen),
      _ => None,
    } {
      self.advance();
      let node = self.parse_expression(); // Recurse back to top
      if !matches!(self.advance(), Some(Token::CloseParen)) {
        // Check for closing parenthesis
        panic!("Parenthesis not closed!");
      }
      return node;
    } else {
      // 3. Probably just LiteralInt left, but keep match for now
      match self.advance() {
        Some(Token::LiteralInt(val)) => return Expr::LiteralInt(*val),
        Some(other) => panic!("Expected expression, found: {}", other),
        _ => panic!("Expected expression, found EOF"),
      }
    }
  }
}
