// #[derive(Default)]
// Base node of AST
pub struct Program {
  pub name: String,
  pub child_functions: Vec<Function>,
  pub pretty_ast: String,
}

impl Program {
  pub fn new(name: String) -> Self {
    Program {
      name: name,
      child_functions: vec![],
      pretty_ast: String::new(),
    }
  }
  pub fn print(&mut self) {
    self
      .pretty_ast
      .push_str(&format!("PROGRAM {}\n", self.name));
    for fun in &self.child_functions {
      self.pretty_ast.push_str(&fun.print());
    }
    println!("{}", self.pretty_ast);
  }
}

// #[derive(Default)]
pub struct Function {
  pub name: String,
  pub child_statements: Vec<Statement>,
}

impl Function {
  pub fn print(&self) -> String {
    let mut func_str = String::new();
    func_str.push_str(&format!("FUNC {}:\n", self.name));
    for stmt in &self.child_statements {
      func_str.push_str(&stmt.print());
    }
    func_str
  }
}

pub enum Statement {
  // DeclareFunction(Keyword, Token),
  Return(Expr),     // return x
  Expression(Expr), // x + 5, !x
}

impl Statement {
  pub fn print(&self) -> String {
    let mut stmt_str = String::new();
    match self {
      Statement::Expression(x) => {
        stmt_str.push_str(&format!("  EXPR[{}]", x.print()));
      }
      Statement::Return(x) => {
        stmt_str.push_str(&format!("  RETURN EXPR[{}] ", x.print()));
      }
    }
    stmt_str
  }
}

// 'Semantic' tokens derived from raw tokens depending on context
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

impl Expr {
  pub fn print(&self) -> String {
    let mut expr_str = String::new();
    match self {
      Expr::LiteralInt(val) => {
        expr_str.push_str(&format!("{}", val));
      }
      Expr::UnOp(op, operand) => match op {
        UnaryOp::Negate => expr_str.push_str(&format!("-<{}>", &operand.print())),
        UnaryOp::BitwiseNot => expr_str.push_str(&format!("~<{}>", &operand.print())),
        UnaryOp::LogicalNot => expr_str.push_str(&format!("!<{}>", &operand.print())),
      },
      Expr::BinOp(op, operand1, operand2) => {
        expr_str.push_str(&format!("({}", &operand1.print()));
        match op {
          BinaryOp::Add => expr_str.push_str(&format!(" + {})", &operand2.print())),
          BinaryOp::Subtract => expr_str.push_str(&format!(" - {})", &operand2.print())),
          BinaryOp::Multiply => expr_str.push_str(&format!(" * {})", &operand2.print())),
          BinaryOp::Divide => expr_str.push_str(&format!(" / {})", &operand2.print())),
        }
      }
    }
    expr_str
  }
}
