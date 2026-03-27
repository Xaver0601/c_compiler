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
  fn peek_logical_and_op(&self) -> Option<LogicalOp> {
    match self.peek() {
      Some(Token::And) => Some(LogicalOp::And),
      _ => None,
    }
  }

  // TODO: think about inlining this
  // Check if next token is a logical OR operator (||)
  fn peek_logical_or_op(&self) -> Option<LogicalOp> {
    match self.peek() {
      Some(Token::Or) => Some(LogicalOp::Or),
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

  // <statement> ::= "return" <exp> ";" | <exp> | "int" <id> [ = <exp> ] ";"
  fn parse_statement(&mut self) -> Statement {
    let token = self.peek().cloned();
    match token {
      // Return statement
      Some(Token::Keyword(Keyword::RETURN)) => {
        self.advance(); // consume 'return'
        let expr = self.parse_expression();
        self.expect(Token::Semicolon, "after return statement");
        return Statement::Return(expr);
      }
      // Variable declaration
      Some(Token::Keyword(Keyword::INT)) => {
        self.advance(); // consume 'int'
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
      // Variable initialization or standalone expression (e.g 2 + 2;)
      Some(_other_token) => {
        let expr = self.parse_expression();
        self.expect(Token::Semicolon, "after expression statement");
        Statement::Expression(expr)
      }
      None => panic!("Expected statement, found EOF (End of File)"),
    }
  }

  // <exp> ::= <id> "=" <exp> | <logical-or-exp>
  fn parse_expression(&mut self) -> Expr {
    if let Some(Token::Identifier(var_name)) = self.peek() {
      let name = var_name.clone();
      if let Some(Token::Assign) = self.tokens.get(self.current + 1) {
        self.advance(); // consume id
        self.advance(); // consume '='
        let expr = self.parse_expression();
        return Expr::Assign(name, Box::new(expr));
      }
    }
    self.parse_logical_or_expression()
  }

  // <logical-or-exp> ::= <logical-and-exp> { "||" <logical-and-exp> }
  fn parse_logical_or_expression(&mut self) -> Expr {
    let mut left = self.parse_logical_and_expression();
    while let Some(op) = self.peek_logical_or_op() {
      self.advance(); // consume the token (||)
      let right = self.parse_logical_and_expression();
      left = Expr::LogOp(op, Box::new(left), Box::new(right));
    }
    left
  }

  // <logical-and-exp> ::= <equality-exp> { "&&" <equality-exp> }
  fn parse_logical_and_expression(&mut self) -> Expr {
    let mut left = self.parse_equality_expression();
    while let Some(op) = self.peek_logical_and_op() {
      self.advance(); // consume the token (&&)
      let right = self.parse_equality_expression();
      left = Expr::LogOp(op, Box::new(left), Box::new(right));
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

  // <factor> ::= "(" <exp> ")" | <unary_op> <factor> | <int> | <id>
  fn parse_factor(&mut self) -> Expr {
    // Try to extract the UnaryOp if the next token is one.
    // "Check if next token is a UnaryOp, if so, pull the value out of it, name that value op and execute the code in the curly brackets
    if let Some(op) = self.peek_unary_op() {
      self.advance();
      let operand = self.parse_factor();
      return Expr::UnOp(op, Box::new(operand));
    }
    match self.peek().cloned() {
      Some(Token::OpenParen) => {
        self.advance();
        let node = self.parse_expression();
        self.expect(Token::CloseParen, "after expression");
        return node;
      }
      Some(Token::LiteralInt(val)) => {
        self.advance();
        return Expr::LiteralInt(val);
      }
      Some(Token::Identifier(var_name)) => {
        self.advance();
        return Expr::Var(var_name.to_string());
      }
      Some(other_token) => panic!("Expected factor, found: {}", other_token),
      None => panic!("Expected factor, found EOF"),
    }
  }
}
