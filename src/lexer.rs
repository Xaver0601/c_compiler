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
// Reads an input file and extracts tokens
pub struct Lexer {
  pub tokens: Vec<Token>,
  pub tokens_literal: Vec<String>,
}

impl Lexer {
  pub fn print_tokens(&self) {
    for tok in &self.tokens {
      println!("{}", tok);
    }
  }
  pub fn print_tokens_literal(&self) {
    for tok in &self.tokens_literal {
      println!("{}", tok);
    }
  }

  pub fn lex(&mut self, path: &String) {
    let content = fs::read_to_string(path).expect("Could not read file");
    self.tokens = Vec::new();
    self.tokens_literal = Vec::new();
    // println!("{content}");
    // (?x) at beginning to ignore whitespaces in regex pattern (good for formatting)
    let re = Regex::new(
      r"(?x)
    (?P<brace_open>\{)    |
    (?P<brace_close>\})   |
    (?P<paren_open>\()    |
    (?P<paren_close>\))   |
    (?P<semicolon>\;)     |
    (?P<kw_int>int)\b     |
    (?P<kw_return>return)\b |
    (?P<ident>[a-zA-Z]\w*) |
    (?P<lit_int>[0-9]+)   |
    (?P<minus>\-)         |
    (?P<tilde>\~)         |
    (?P<excl>\!)          |
    (?P<plus>\+)          |
    (?P<star>\*)          |
    (?P<slash>\/)",
    )
    .unwrap();

    for cap in re.captures_iter(&content) {
      let matched_text = &cap[0];
      self.tokens_literal.push(String::from(matched_text));

      let mut group_names = re.capture_names().flatten(); // Iterator for all group names
      let group_name = group_names.find(|name| cap.name(name).is_some()); // Go over iterator and check for each element if it matches the group name found in cap
      if let Some(found_name) = group_name {
        let token = match found_name {
          "brace_open" => Token::OpenBrace,
          "brace_close" => Token::CloseBrace,
          "paren_open" => Token::OpenParen,
          "paren_close" => Token::CloseParen,
          "semicolon" => Token::Semicolon,
          "kw_int" => Token::Keyword(Keyword::INT),
          "kw_return" => Token::Keyword(Keyword::RETURN),
          "ident" => Token::Identifier(cap[0].to_string()),
          "lit_int" => Token::LiteralInt(cap[0].parse().expect("Not a number")),
          "minus" => Token::Minus,
          "tilde" => Token::Tilde,
          "exl" => Token::Exclamation,
          "plus" => Token::Plus,
          "star" => Token::Star,
          "slash" => Token::Slash,
          _ => panic!("Unknown token name: {}", found_name),
        };
        self.tokens.push(token);
      }
    }
  }
}
