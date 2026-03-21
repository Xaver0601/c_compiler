use regex::Regex;
use std::fmt;
use std::fs;

// Literal 'raw' tokens
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
  LiteralInt(i32),
  Identifier(String),
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Keyword {
  INT,
  #[default] // TODO: change this default
  RETURN,
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
      Token::LiteralInt(n) => write!(f, "INT({})", n),
      // _ => write!(f, "NOT IMPLEMENTED"),
    }
  }
}

#[derive(Default)]
pub struct Lexer {
  pub tokens: Vec<Token>,
  pub tokens_string: Vec<String>,
}

impl Lexer {
  pub fn print_tokens(&self) {
    for tok in &self.tokens {
      println!("{}", tok);
    }
  }
  pub fn print_tokens_string(&self) {
    for tok in &self.tokens_string {
      println!("{}", tok);
    }
  }

  pub fn lex(&mut self, path: &String) {
    let content = fs::read_to_string(path).expect("Could not read file");
    // println!("{content}");
    let re = Regex::new(
    r"(\{)|(\})|(\()|(\))|(\;)|(int)\b|(return)\b|([a-zA-Z]\w*)|([0-9]+)|(\-)|(\~)|(\!)|(\+)|(\*)|(\/)",
  )
  .unwrap();
    self.tokens = Vec::new();
    self.tokens_string = Vec::new();
    for cap in re.captures_iter(&content) {
      self.tokens_string.push(String::from(&cap[0]));
      let token = if cap.get(1).is_some() {
        Token::OpenBrace
      } else if cap.get(2).is_some() {
        Token::CloseBrace
      } else if cap.get(3).is_some() {
        Token::OpenParen
      } else if cap.get(4).is_some() {
        Token::CloseParen
      } else if cap.get(5).is_some() {
        Token::Semicolon
      } else if cap.get(6).is_some() {
        Token::Keyword(Keyword::INT)
      } else if cap.get(7).is_some() {
        Token::Keyword(Keyword::RETURN)
      } else if let Some(m) = cap.get(8) {
        Token::Identifier(m.as_str().to_string())
      } else if let Some(m) = cap.get(9) {
        Token::LiteralInt(m.as_str().parse().expect("Not a number"))
      } else if cap.get(10).is_some() {
        Token::Minus
      } else if cap.get(11).is_some() {
        Token::Tilde
      } else if cap.get(12).is_some() {
        Token::Exclamation
      } else if cap.get(13).is_some() {
        Token::Plus
      } else if cap.get(14).is_some() {
        Token::Star
      } else if cap.get(15).is_some() {
        Token::Slash
      } else {
        continue;
      };
      self.tokens.push(token);
    }
  }
}
