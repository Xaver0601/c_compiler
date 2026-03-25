use crate::ast;

// Converts an AST program into assembly
pub struct Generator {
  pub ast: ast::Program,
  pub jump_counter: i32,
}

impl Generator {
  pub fn generate_program(&mut self) -> String {
    let mut assembly = String::new();
    for fun in &self.ast.child_functions {
      assembly += &format!(".globl {name}\n{name}:\n", name = fun.name);
      assembly += &format!("{}", Self::generate_statement(&mut self.jump_counter, fun));
    }
    assembly += "ret\n.section .note.GNU-stack,\"\",@progbits\n";
    assembly
  }

  fn generate_statement(jump_counter: &mut i32, fun: &ast::Function) -> String {
    let mut stmt = String::new();
    for stm in &fun.child_statements {
      match stm {
        ast::Statement::Return(x) => {
          stmt += &Self::generate_expression(jump_counter, x);
          // stmt += "movl %eax, %eax\n";
        }
        ast::Statement::Expression(x) => {
          stmt += &Self::generate_expression(jump_counter, x);
        }
        ast::Statement::Declare(_var, x) => {
          // TODO: implement assembly
          if x.is_some() {
            println!("Local variables not implemented yet")
            // stmt += &Self::generate_expression(jump_counter, x.as_ref().unwrap());
          } else {
            println!("Local variables not implemented yet");
            // stmt += &Self::generate_expression(jump_counter, x.as_ref().unwrap());
          }
        }
      }
    }
    stmt
  }

  fn generate_expression(jump_counter: &mut i32, expr: &ast::Expr) -> String {
    match expr {
      ast::Expr::LiteralInt(val) => format!("movl ${}, %eax\n", val),
      ast::Expr::UnOp(op, operand) => {
        let mut asm = String::new();
        asm += &Self::generate_expression(jump_counter, operand);
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
            asm += "sete %al\n"; // sete sets value to 1 if ZF flag (zero flag) is 1; can only modify 1 byte, so we use %al (lower byte of %eax)
          }
        }
        asm
      }
      ast::Expr::BinOp(op, operand1, operand2) => {
        let mut asm = String::new();
        asm += &Self::generate_expression(jump_counter, operand1);
        asm += "push %rax\n";
        asm += &Self::generate_expression(jump_counter, operand2); // 2nd operand in eax
        asm += "pop %rcx\n"; // 1st operand in ecx
        match op {
          &ast::BinaryOp::Add => {
            asm += "addl %ecx, %eax\n"; // addl [src, dst]: src + dst, saves result in dst
          }
          &ast::BinaryOp::Subtract => asm += "subl %eax, %ecx\nmovl %ecx, %eax\n", // subl [src, dst]: dst - src, saves result in dst
          &ast::BinaryOp::Multiply => asm += "imull %ecx, %eax\n", // imull [src, dst]: src * dst, saves result in dst
          &ast::BinaryOp::Divide => {
            asm += "push %rcx\nmovl %eax, %ecx\npop %rax\n"; // Swap: 1st operand in eax, 2nd in ecx
            asm += "cdq\n"; // Converts eax into edx:eax by sign extension
            asm += "idivl %ecx\n"; // eax / ecx, eax holds the quotient, edx the remainder
          }
          &ast::BinaryOp::Equal => {
            // Similar to LogicalNot
            asm += "cmpl %eax, %ecx\n";
            asm += "movl $0, %eax\n";
            asm += "sete %al\n";
          }
          &ast::BinaryOp::Unequal => {
            asm += "cmpl %eax, %ecx\n";
            asm += "movl $0, %eax\n";
            asm += "setne %al\n";
          }
          // These use the SF flag (sign flag)
          &ast::BinaryOp::Less => {
            asm += "cmpl %eax, %ecx\n";
            asm += "movl $0, %eax\n";
            asm += "setl %al\n";
          }
          &ast::BinaryOp::LessEqual => {
            asm += "cmpl %eax, %ecx\n";
            asm += "movl $0, %eax\n";
            asm += "setle %al\n";
          }
          &ast::BinaryOp::Greater => {
            asm += "cmpl %eax, %ecx\n";
            asm += "movl $0, %eax\n";
            asm += "setg %al\n";
          }
          &ast::BinaryOp::GreaterEqual => {
            asm += "cmpl %eax, %ecx\n";
            asm += "movl $0, %eax\n";
            asm += "setge %al\n";
          }
          &ast::BinaryOp::Or => {
            asm += "cmpl $0, %ecx\n"; // If 1st operand is zero set ZF flag
            asm += &format!("je _clause2_{}\n", jump_counter); // If e1 was zero, check e2
            asm += "movl $1, %eax\n"; // If 1st operand is not zero we already know that e1 || e2 will evaluate to 1 (short-circuiting)
            asm += &format!("jmp _end_{}\n", jump_counter); // Return 1 and skip e2
            asm += &format!("_clause2_{}:\n", jump_counter);
            asm += "cmpl $0, %eax\n"; // Check if e2 is zero
            asm += "movl $0, %eax\n";
            asm += "setne %al\n"; // If it wasn't, return 1
            asm += &format!("_end_{}:\n", jump_counter);
            *jump_counter += 1;
          }
          &ast::BinaryOp::And => {
            asm += "cmpl $0, %ecx\n"; // If 1st operand is zero set ZF flag
            asm += &format!("jne _clause2_{}\n", jump_counter); // If e1 was not zero, check e2
            asm += "movl $0, %eax\n"; // If 1st operand is zero we already know that e1 && e2 will evaluate to 0 (short-circuiting)
            asm += &format!("jmp _end_{}\n", jump_counter); // Return 0 and skip e2
            asm += &format!("_clause2_{}:\n", jump_counter);
            asm += "cmpl $0, %eax\n"; // Check if e2 is zero
            asm += "movl $0, %eax\n";
            asm += "setne %al\n"; // If it wasn't, return 1
            asm += &format!("_end_{}:\n", jump_counter);
            *jump_counter += 1;
          }
        }
        asm
      }
      // TODO: implement assembly
      ast::Expr::Assign(_var_name, _operand) => format!("movl $0, %eax\n"),
    }
  }
}
