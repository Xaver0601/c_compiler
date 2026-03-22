use crate::ast;

// Converts an AST program into assembly
pub struct Generator {
  pub ast: ast::Program,
}

impl Generator {
  pub fn generate_program(&self) -> String {
    let mut assembly = String::new();
    for fun in &self.ast.child_functions {
      assembly += &format!(".globl {name}\n{name}:\n", name = fun.name);
      assembly += &format!("{}", Self::generate_statement(&fun));
    }
    assembly += "ret\n.section .note.GNU-stack,\"\",@progbits\n";
    assembly
  }

  fn generate_statement(fun: &ast::Function) -> String {
    let mut stmt = String::new();
    for stm in &fun.child_statements {
      match stm {
        ast::Statement::Return(x) => {
          stmt += &Self::generate_expression(x);
          // stmt += "movl %eax, %eax\n";
        }
        ast::Statement::Expression(x) => {
          stmt += &Self::generate_expression(x);
        }
      }
    }
    stmt
  }

  fn generate_expression(expr: &ast::Expr) -> String {
    match expr {
      ast::Expr::LiteralInt(val) => format!("movl ${}, %eax\n", val),
      ast::Expr::UnOp(op, operand) => {
        let mut asm = String::new();
        asm += &Self::generate_expression(operand);
        match op {
          ast::UnaryOp::Negate => {
            asm += "negl %eax\n";
          }
          ast::UnaryOp::BitwiseNot => {
            asm += "notl %eax\n";
          }
          ast::UnaryOp::LogicalNot => {
            asm += "cmpl $0, %eax\n"; // if (%eax - 0) == 0 this sets the ZF flag to 1
            asm += "movl $0, %eax\n"; // zero out %eax
            asm += "sete %al\n"; // sete sets value to 1 if ZF flag is 1; can only modify 1 byte, so we use %al (lower byte of %eax)
          }
        }
        asm
      }
      ast::Expr::BinOp(op, operand1, operand2) => {
        let mut asm = String::new();
        asm += &Self::generate_expression(operand1);
        asm += "push %rax\n";
        asm += &Self::generate_expression(operand2); // 2nd operand in eax
        asm += "pop %rcx\n"; // 1st operand in ecx
        match op {
          ast::BinaryOp::Add => {
            asm += "addl %ecx, %eax\n"; // addl [src, dst]: src + dst, saves result in dst
          }
          ast::BinaryOp::Subtract => asm += "subl %eax, %ecx\nmovl %ecx, %eax\n", // subl [src, dst]: dst - src, saves result in dst
          ast::BinaryOp::Multiply => asm += "imull %ecx, %eax\n", // imull [src, dst]: src * dst, saves result in dst
          ast::BinaryOp::Divide => {
            asm += "push %rcx\nmovl %eax, %ecx\npop %rax\n"; // Swap: 1st operand in eax, 2nd in ecx
            asm += "cdq\n"; // Converts eax into edx:eax by sign extension
            asm += "idivl %ecx\n"; // eax / ecx, eax holds the quotient, edx the remainder
          }
        }
        asm
      }
    }
  }
}
