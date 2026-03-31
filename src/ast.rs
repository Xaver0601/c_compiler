// #[derive(Default)]
// Base node of AST
pub struct Program {
  pub name: String,
  pub child_functions: Vec<Function>,
  pub pretty_ast: String,
}

// #[derive(Default)]
pub struct Function {
  pub name: String,
  pub child_block_items: Vec<BlockItem>,
}

pub enum BlockItem {
  Stmt(Statement),
  Decl(String, Option<Expression>), // int a (= 5);
}

pub enum Statement {
  Ret(Expression),                                          // return x
  Expr(Expression), // x + 5, !x  TODO: make this Option<Expression> to allow null statements ';'
  Cond(Expression, Box<Statement>, Option<Box<Statement>>), // if(expr) {statement1} (else {statement2})
  Compound(Vec<BlockItem>),
  #[rustfmt::skip]  // Prevent formatter from line breaking
  For(Option<Expression>, Expression, Option<Expression>, Box<Statement>), // for(a=1; a < 10; a = a + 1) {stmt}, for(;1;) {stmt}
  #[rustfmt::skip]
  ForDecl(String, Option<Expression>, Expression, Option<Expression>, Box<Statement>), // for(int a=1; a < 10; a = a + 1) {stmt}
  While(Expression, Box<Statement>), // while(x < 5) {stmt}
  Do(Box<Statement>, Expression),    // do {stmt} while(x < 5)
  Break,                             // break
  Continue,                          // continue
}

pub enum Expression {
  LiteralInt(i32),
  UnOp(UnaryOp, Box<Expression>),
  BinOp(BinaryOp, Box<Expression>, Box<Expression>),
  LogOp(LogicalOp, Box<Expression>, Box<Expression>),
  Assign(String, Box<Expression>),
  Var(String),
  Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
}

// 'Semantic' tokens derived from raw tokens depending on context
#[derive(Clone, Copy)]
pub enum UnaryOp {
  Negate,     // -
  BitwiseNot, // ~
  LogicalNot, // !
}

// Ordered by precedence
#[derive(Clone, Copy)]
pub enum BinaryOp {
  Multiply,     // *
  Divide,       // /
  Add,          // +
  Subtract,     // -
  Less,         // <
  LessEqual,    // <=
  Greater,      // >
  GreaterEqual, // >=
  Equal,        // ==
  Unequal,      // !=
}

// Logical operators than can short-circuit
#[derive(Clone, Copy)]
pub enum LogicalOp {
  And, // &&
  Or,  // ||
}

impl Program {
  pub fn new(name: String) -> Self {
    Program {
      name: name,
      child_functions: vec![],
      pretty_ast: String::new(),
    }
  }

  pub fn print(&self) {
    println!("{}", self.pretty_ast);
  }

  pub fn build_pretty_ast(&mut self) {
    self
      .pretty_ast
      .push_str(&format!("PROGRAM {}\n", self.name));
    for fun in &self.child_functions {
      self.pretty_ast.push_str(&fun.print());
    }
  }
}

impl Function {
  pub fn print(&self) -> String {
    let mut func_str = String::new();
    func_str.push_str(&format!("FUNC {}:\n", self.name));
    for block_item in &self.child_block_items {
      func_str.push_str(&block_item.print());
    }
    func_str
  }
}

impl BlockItem {
  pub fn print(&self) -> String {
    let mut stmt_str = String::new();
    match self {
      BlockItem::Stmt(x) => {
        stmt_str.push_str(&format!("{}", x.print()));
      }
      BlockItem::Decl(var, x) => {
        let temp_str = &mut format!("  DECLARE VAR[{}]", var);
        if x.is_some() {
          temp_str.push_str(&format!(" = EXPR[{}]", x.as_ref().unwrap().print()));
        }
        stmt_str.push_str(temp_str);
      }
    }
    stmt_str.push('\n');
    stmt_str
  }
}

impl Statement {
  pub fn print(&self) -> String {
    let mut stmt_str = String::new();
    match self {
      Statement::Expr(x) => {
        stmt_str.push_str(&format!("  EXPR[{}]", x.print()));
      }
      Statement::Ret(x) => {
        stmt_str.push_str(&format!("  RETURN EXPR[{}]", x.print()));
      }
      Statement::Cond(x, a, b) => {
        stmt_str.push_str(&format!("  IF ({})\n", x.print()));
        stmt_str.push_str(&format!("  {}", a.print()));
        if b.is_some() {
          stmt_str.push_str(&format!("\n  ELSE\n  {}", b.as_ref().unwrap().print()));
        }
      }
      Statement::Compound(x) => {
        stmt_str.push_str(&format!("{{\n"));
        for block in x {
          stmt_str.push_str(&format!("  {}", block.print()));
        }
        stmt_str.push_str(&format!("  }}"));
      }
      Statement::For(init, cond, post, stm) => {
        stmt_str.push_str(&format!("FOR("));
        if init.is_some() {
          stmt_str.push_str(&format!("{}", init.as_ref().unwrap().print()));
        }
        stmt_str.push_str(&format!(";"));
        stmt_str.push_str(&format!("{}", cond.print()));
        stmt_str.push_str(&format!(";"));
        if post.is_some() {
          stmt_str.push_str(&format!("{}", post.as_ref().unwrap().print()));
        }
        stmt_str.push_str(&format!(") {{\n"));
        stmt_str.push_str(&format!("{}", stm.print()));
        stmt_str.push_str(&format!("}}\n"));
      }
      Statement::ForDecl(var, init, cond, post, stm) => {
        stmt_str.push_str(&format!("FOR(DECLARE {}", var));
        if init.is_some() {
          stmt_str.push_str(&format!(" = {}", init.as_ref().unwrap().print()));
        }
        stmt_str.push_str(&format!(";"));
        stmt_str.push_str(&format!("{}", cond.print()));
        stmt_str.push_str(&format!(";"));
        if post.is_some() {
          stmt_str.push_str(&format!("{}", post.as_ref().unwrap().print()));
        }
        stmt_str.push_str(&format!(") {{\n"));
        stmt_str.push_str(&format!("  {}", stm.print()));
        stmt_str.push_str(&format!("}}\n"));
      }
      Statement::While(cond, stm) => {
        stmt_str.push_str(&format!("WHILE({}){{\n", cond.print()));
        stmt_str.push_str(&format!("  {}}}\n", stm.print()));
      }
      Statement::Do(stm, cond) => {
        stmt_str.push_str(&format!("DO{{\n  {}\n}}\n", stm.print()));
        stmt_str.push_str(&format!("WHILE({})\n", cond.print()));
      }
      Statement::Break => {
        stmt_str.push_str(&format!("BREAK"));
      }
      Statement::Continue => {
        stmt_str.push_str(&format!("CONTINUE"));
      }
    }
    // stmt_str.push('\n');
    stmt_str
  }
}

impl Expression {
  pub fn print(&self) -> String {
    let mut expr_str = String::new();
    match self {
      Expression::LiteralInt(val) => {
        expr_str.push_str(&format!("{}", val));
      }
      Expression::UnOp(op, operand) => match op {
        UnaryOp::Negate => expr_str.push_str(&format!("-<{}>", &operand.print())),
        UnaryOp::BitwiseNot => expr_str.push_str(&format!("~<{}>", &operand.print())),
        UnaryOp::LogicalNot => expr_str.push_str(&format!("!<{}>", &operand.print())),
      },
      Expression::BinOp(op, operand1, operand2) => {
        expr_str.push_str(&format!("({}", &operand1.print()));
        match op {
          BinaryOp::Add => expr_str.push_str(&format!(" + {})", &operand2.print())),
          BinaryOp::Subtract => expr_str.push_str(&format!(" - {})", &operand2.print())),
          BinaryOp::Multiply => expr_str.push_str(&format!(" * {})", &operand2.print())),
          BinaryOp::Divide => expr_str.push_str(&format!(" / {})", &operand2.print())),
          BinaryOp::Less => expr_str.push_str(&format!(" < {})", &operand2.print())),
          BinaryOp::LessEqual => expr_str.push_str(&format!(" <= {})", &operand2.print())),
          BinaryOp::Greater => expr_str.push_str(&format!(" > {})", &operand2.print())),
          BinaryOp::GreaterEqual => expr_str.push_str(&format!(" >= {})", &operand2.print())),
          BinaryOp::Equal => expr_str.push_str(&format!(" == {})", &operand2.print())),
          BinaryOp::Unequal => expr_str.push_str(&format!(" != {})", &operand2.print())),
        }
      }
      Expression::LogOp(op, operand1, operand2) => {
        expr_str.push_str(&format!("({}", &operand1.print()));
        match op {
          LogicalOp::And => expr_str.push_str(&format!(" && {})", &operand2.print())),
          LogicalOp::Or => expr_str.push_str(&format!(" || {})", &operand2.print())),
        }
      }
      Expression::Assign(var_name, operand) => {
        expr_str.push_str(&format!("{} = {}", var_name, &operand.print()));
      }
      Expression::Var(var_name) => {
        expr_str.push_str(&format!("{}", var_name));
      }
      Expression::Ternary(x, a, b) => {
        expr_str.push_str(&format!("{} ? {} : {}", x.print(), a.print(), b.print()));
      }
    }
    expr_str
  }
}
