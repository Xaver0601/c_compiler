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

  // Throw an error if next token is not the expected token, consumes next token
  // A 'assert!(match!(exp_token, found_token.unwrap()))' will not work because found_token is potentially None
  fn expect(&mut self, exp_token: Token, error: &str) {
    match self.advance() {
      Some(found) if found == &exp_token => {}
      Some(other) => {
        panic!("Expected '{}' {}, found: {}", exp_token, error, other)
      }
      None => {
        panic!(
          "Expected '{}' {}, found: end of file (EOF)",
          exp_token, error
        )
      }
    }
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

  // Check if next token is a relational operator (<, <=, >, >=)
  fn peek_relational_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::Less) => Some(BinaryOp::Less),
      Some(Token::LessEqual) => Some(BinaryOp::LessEqual),
      Some(Token::Greater) => Some(BinaryOp::Greater),
      Some(Token::GreaterEqual) => Some(BinaryOp::GreaterEqual),
      _ => None,
    }
  }

  // Check if next token is a equality operator (!=, ==)
  fn peek_equality_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::Equal) => Some(BinaryOp::Equal),
      Some(Token::Unequal) => Some(BinaryOp::Unequal),
      _ => None,
    }
  }

  // TODO: think about inlining this
  // Check if next token is a logical AND operator (&&)
  fn peek_logical_and_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::And) => Some(BinaryOp::And),
      _ => None,
    }
  }

  // TODO: think about inlining this
  // Check if next token is a logical OR operator (||)
  fn peek_logical_or_op(&self) -> Option<BinaryOp> {
    match self.peek() {
      Some(Token::Or) => Some(BinaryOp::Or),
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
    self.expect(Token::Keyword(Keyword::INT), "for function type");

    // Expect function name
    let name = match self.advance() {
      Some(Token::Identifier(n)) => n.clone(),
      _ => panic!("Expected function name"),
    };

    self.expect(Token::OpenParen, "for function parameters");
    self.expect(Token::CloseParen, "for function parameters");
    self.expect(Token::OpenBrace, "for function start");

    // Parse the inner statements
    let mut statements = Vec::new();
    while !matches!(self.peek(), Some(Token::CloseBrace)) {
      statements.push(self.parse_statement());
      // If the last statement was a 'return' the function ends
      if matches!(statements.last(), Some(Statement::Return(_x))) {
        break;
      }
    }

    self.expect(Token::CloseBrace, "for function end");

    Function {
      name,
      child_statements: statements,
    }
  }

  // <statement> ::= "return" <exp> ";"
  fn parse_statement(&mut self) -> Statement {
    let token = self.advance().clone();
    match token {
      // Return statement
      Some(Token::Keyword(Keyword::RETURN)) => {
        let expr = self.parse_expression();
        self.expect(Token::Semicolon, "after return statement");
        return Statement::Return(expr);
      }
      // Variable declaration
      Some(Token::Keyword(Keyword::INT)) => {
        let var_name = match self.advance() {
          Some(Token::Identifier(n)) => n.clone(),
          Some(other) => panic!("Expected variable name (string), found: {}", other),
          None => panic!("Expected variable name (string) in declaration, found EOF (End of File)"),
        };
        match self.advance() {
          Some(Token::Semicolon) => return Statement::Declare(var_name, None), // just declaration
          Some(Token::Assign) => {
            // Declaration with initialization
            let expr = self.parse_expression();
            self.expect(Token::Semicolon, "after variable initialization");
            return Statement::Declare(var_name, Some(expr));
          }
          Some(other) => panic!("Expected ';' after statement, found: {}", other),
          None => panic!("Expected ';' after statement, found EOF (End of File)"),
        }
      }
      Some(Token::Identifier(var_name)) => {
        let v_name = var_name.clone();
        self.expect(Token::Assign, "in variable assignment");
        let expr = self.parse_expression();
        self.expect(Token::Semicolon, "after variable assignment");
        return Statement::Declare(v_name, Some(expr)); // Assignment
      }
      Some(other_token) => panic!("Unsupported statement starting with token: {}", other_token),
      _ => panic!("Unsupported keyword"),
    };
  }

  // <exp> ::= <logical-and-exp> { "||" <logical-and-exp> }
  fn parse_expression(&mut self) -> Expr {
    let mut left = self.parse_logical_and_expression();
    while let Some(op) = self.peek_logical_or_op() {
      self.advance(); // consume the token (||)
      let right = self.parse_logical_and_expression();
      left = Expr::BinOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <logical-and-exp> ::= <equality-exp> { "&&" <equality-exp> }
  fn parse_logical_and_expression(&mut self) -> Expr {
    let mut left = self.parse_equality_expression();
    while let Some(op) = self.peek_logical_and_op() {
      self.advance(); // consume the token (&&)
      let right = self.parse_equality_expression();
      left = Expr::BinOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <equality-exp> ::= <relational-exp> { ("!=" | "==") <relational-exp> }
  fn parse_equality_expression(&mut self) -> Expr {
    let mut left = self.parse_relational_expression();
    while let Some(op) = self.peek_equality_op() {
      self.advance(); // consume the token (!=, ==)
      let right = self.parse_relational_expression();
      left = Expr::BinOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <relational-exp> ::= <additive-exp> { ("<" | ">" | "<=" | ">=") <additive-exp> }
  fn parse_relational_expression(&mut self) -> Expr {
    let mut left = self.parse_additive_expression();
    while let Some(op) = self.peek_relational_op() {
      self.advance(); // consume the token (<, <=, >, >=)
      let right = self.parse_additive_expression();
      left = Expr::BinOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <additive_exp> ::= <term> { ("+" | "-") <term> }
  fn parse_additive_expression(&mut self) -> Expr {
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
      // Check for closing parenthesis
      if !matches!(self.advance(), Some(Token::CloseParen)) {
        panic!("Parenthesis not closed!");
      }
      return node;
    } else {
      // TODO: this probably needs to be updated for variable declaration
      // 3. Probably just LiteralInt left, but keep match for now
      match self.advance() {
        Some(Token::LiteralInt(val)) => return Expr::LiteralInt(*val),
        Some(other) => panic!("Expected expression, found: {}", other),
        _ => panic!("Expected expression, found EOF"),
      }
    }
  }
}
