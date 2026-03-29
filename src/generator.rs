use crate::ast;

// Converts an AST program into assembly
pub struct Generator {
  pub ast: ast::Program,
  pub jump_counter: i32,
  // pub var_map: std::collections::HashMap<String, i32>, // could maybe be used for global variables
  pub stack_index: i32,
}

impl Generator {
  pub fn generate_program(&mut self) -> String {
    let mut assembly = String::new();
    for fun in &self.ast.child_functions {
      assembly += &format!(".globl {name}\n{name}:\n", name = fun.name); // Function label
      assembly += "  push %rbp\n  mov %rsp, %rbp\n"; // Function prologue
      assembly += &format!(
        "{}",
        Self::generate_statement(
          fun,
          &mut self.jump_counter,
          std::collections::HashMap::new(), // Each function (scope) gets a blank variable hashmap
          &mut self.stack_index
        )
      );
      // If a function has no return statement, return 0
      let mut has_return: bool = false;
      for block_item in &fun.child_block_items {
        if matches!(block_item, ast::BlockItem::Stmt(ast::Statement::Ret(_x))) {
          has_return = true;
          break;
        }
      }
      if !has_return {
        assembly += "  movl $0, %eax\n";
      }
      assembly += "  mov %rbp, %rsp\n  pop %rbp\n  ret\n"; // Function epilogue
    }
    assembly += ".section .note.GNU-stack,\"\",@progbits\n"; // Metadata so gcc doesn't throw a warning
    assembly
  }

  // TODO: implement this
  // fn generate_block_item() -> String {
  //   let block_item = String::new();
  //   block_item
  // }

  fn generate_statement(
    fun: &ast::Function,
    jump_counter: &mut i32,
    mut var_map: std::collections::HashMap<String, i32>,
    stack_index: &mut i32,
  ) -> String {
    let mut stmt = String::new();
    for stm in &fun.child_block_items {
      match stm {
        ast::BlockItem::Stmt(ast::Statement::Ret(x)) => {
          stmt += &Self::generate_expression(x, jump_counter, &var_map);
          // stmt += "movl %eax, %eax\n";
        }
        ast::BlockItem::Stmt(ast::Statement::Expr(x)) => {
          stmt += &Self::generate_expression(x, jump_counter, &var_map);
        }
        ast::BlockItem::Decl(var, x) => {
          if var_map.contains_key(var) {
            panic!("Variable '{}' has already been declared", var);
          }
          if x.is_some() {
            stmt += &Self::generate_expression(x.as_ref().unwrap(), jump_counter, &var_map);
            stmt += "  push %rax\n"; // Push variable onto stack
          } else {
            stmt += "  movl $0, %eax\n"; // Init 'int a;' as a = 0
            stmt += "  push %rax\n";
          }
          var_map.insert(var.clone(), *stack_index);
          *stack_index -= 8; // This will give each variable 8 bytes of space
        }
        ast::BlockItem::Stmt(ast::Statement::Cond(_x, _a, _b)) => {
          // TODO: implement this
          // stmt += &Self::generate_expression(x, jump_counter, &var_map);
        }
      }
    }
    stmt
  }

  fn generate_expression(
    expr: &ast::Expression,
    jump_counter: &mut i32,
    var_map: &std::collections::HashMap<String, i32>,
  ) -> String {
    let mut asm = String::new();
    match expr {
      ast::Expression::LiteralInt(val) => {
        asm += &format!("  movl ${}, %eax\n", val);
        asm
      }
      ast::Expression::UnOp(op, operand) => {
        asm += &Self::generate_expression(operand, jump_counter, var_map);
        match op {
          ast::UnaryOp::Negate => {
            asm += "  negl %eax\n";
          }
          ast::UnaryOp::BitwiseNot => {
            asm += "  notl %eax\n";
          }
          ast::UnaryOp::LogicalNot => {
            asm += "  cmpl $0, %eax\n"; // if (%eax - 0) == 0 this sets the ZF flag to 1
            asm += "  movl $0, %eax\n"; // zero out %eax
            asm += "  sete %al\n"; // sete sets value to 1 if ZF flag (zero flag) is 1; can only modify 1 byte, so we use %al (lower byte of %eax)
          }
        }
        asm
      }
      ast::Expression::BinOp(op, operand1, operand2) => {
        asm += &Self::generate_expression(operand1, jump_counter, var_map);
        asm += "  push %rax\n";
        asm += &Self::generate_expression(operand2, jump_counter, var_map); // 2nd operand in eax
        asm += "  pop %rcx\n"; // 1st operand in ecx
        match op {
          &ast::BinaryOp::Add => {
            asm += "  addl %ecx, %eax\n"; // addl [src, dst]: src + dst, saves result in dst
          }
          &ast::BinaryOp::Subtract => asm += "  subl %eax, %ecx\n  movl %ecx, %eax\n", // subl [src, dst]: dst - src, saves result in dst
          &ast::BinaryOp::Multiply => asm += "  imull %ecx, %eax\n", // imull [src, dst]: src * dst, saves result in dst
          &ast::BinaryOp::Divide => {
            asm += "  push %rcx\n  movl %eax, %ecx\n  pop %rax\n"; // Swap: 1st operand in eax, 2nd in ecx
            asm += "  cdq\n"; // Converts eax into edx:eax by sign extension
            asm += "  idivl %ecx\n"; // eax / ecx, eax holds the quotient, edx the remainder
          }
          &ast::BinaryOp::Equal => {
            // Similar to LogicalNot
            asm += "  cmpl %eax, %ecx\n";
            asm += "  movl $0, %eax\n";
            asm += "  sete %al\n";
          }
          &ast::BinaryOp::Unequal => {
            asm += "  cmpl %eax, %ecx\n";
            asm += "  movl $0, %eax\n";
            asm += "  setne %al\n";
          }
          // These use the SF flag (sign flag)
          &ast::BinaryOp::Less => {
            asm += "  cmpl %eax, %ecx\n";
            asm += "  movl $0, %eax\n";
            asm += "  setl %al\n";
          }
          &ast::BinaryOp::LessEqual => {
            asm += "  cmpl %eax, %ecx\n";
            asm += "  movl $0, %eax\n";
            asm += "  setle %al\n";
          }
          &ast::BinaryOp::Greater => {
            asm += "  cmpl %eax, %ecx\n";
            asm += "  movl $0, %eax\n";
            asm += "  setg %al\n";
          }
          &ast::BinaryOp::GreaterEqual => {
            asm += "  cmpl %eax, %ecx\n";
            asm += "  movl $0, %eax\n";
            asm += "  setge %al\n";
          }
        }
        asm
      }
      // LogOp is distinct from BinaryOp because we may not evaluate e2 before we checked if e1 already fulfilled the condition
      ast::Expression::LogOp(op, operand1, operand2) => {
        asm += &Self::generate_expression(operand1, jump_counter, var_map);

        // Claim the current ID and increment immediately for any nested expressions
        let current_jump = *jump_counter;
        *jump_counter += 1;

        asm += "  push %rax\n";
        asm += "  pop %rcx\n"; // 1st operand in ecx
        match op {
          &ast::LogicalOp::Or => {
            asm += "  cmpl $0, %ecx\n"; // If 1st operand is zero set ZF flag
            asm += &format!("  je _clause2_{}\n", current_jump); // If e1 was zero, check e2
            asm += "  movl $1, %eax\n"; // If 1st operand is not zero we already know that e1 || e2 will evaluate to 1 (short-circuiting)
            asm += &format!("  jmp _end_{}\n", current_jump); // Return 1 and skip e2
            asm += &format!("  _clause2_{}:\n", current_jump);
            asm += &Self::generate_expression(operand2, jump_counter, var_map); // 2nd operand in eax
            asm += "  cmpl $0, %eax\n"; // Check if e2 is zero
            asm += "  movl $0, %eax\n";
            asm += "  setne %al\n"; // If it wasn't, return 1
            asm += &format!("  _end_{}:\n", current_jump);
          }
          &ast::LogicalOp::And => {
            asm += "  cmpl $0, %ecx\n"; // If 1st operand is zero set ZF flag
            asm += &format!("  jne _clause2_{}\n", current_jump); // If e1 was not zero, check e2
            asm += "  movl $0, %eax\n"; // If 1st operand is zero we already know that e1 && e2 will evaluate to 0 (short-circuiting)
            asm += &format!("  jmp _end_{}\n", current_jump); // Return 0 and skip e2
            asm += &format!("  _clause2_{}:\n", current_jump);
            asm += &Self::generate_expression(operand2, jump_counter, var_map); // 2nd operand in eax
            asm += "  cmpl $0, %eax\n"; // Check if e2 is zero
            asm += "  movl $0, %eax\n";
            asm += "  setne %al\n"; // If it wasn't, return 1
            asm += &format!("  _end_{}:\n", current_jump);
          }
        }
        asm
      }
      ast::Expression::Assign(var_name, operand) => {
        asm += &Self::generate_expression(operand, jump_counter, var_map);
        let var_offset = var_map.get(var_name);
        if var_offset.is_some() {
          asm += &format!("  movl %eax, {}(%rbp)\n", var_offset.unwrap());
          asm
        } else {
          panic!(
            "Assigning variable '{}' which has not been declared yet",
            var_name
          )
        }
      }
      ast::Expression::Var(var_name) => {
        let var_offset = var_map.get(var_name);
        if var_offset.is_some() {
          // TODO: This will put put the variables value as return value (e.g. when function has no return)
          asm += &format!("  movl {}(%rbp), %eax\n", var_offset.unwrap());
          asm
        } else {
          panic!(
            "Assigning variable '{}' which has not been declared yet",
            var_name
          )
        }
      }
      ast::Expression::Ternary(x, a, b) => {
        asm += &Self::generate_expression(x, jump_counter, var_map);

        let current_jump = *jump_counter;
        *jump_counter += 1;

        asm += &format!("  cmpl $0, %eax\n");
        asm += &format!("  je _ternary_{}\n", current_jump);
        asm += &Self::generate_expression(a, jump_counter, var_map);
        asm += &format!("  jmp _post_ternary_{}\n", current_jump);
        asm += &format!("_ternary_{}:\n", current_jump);
        asm += &Self::generate_expression(b, jump_counter, var_map);
        asm += &format!("_post_ternary_{}:\n", current_jump);
        asm
      }
    }
  }
}
