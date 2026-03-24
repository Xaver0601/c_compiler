# Basic Compiler for a subset of C
**This implementation follows the** [Writing a C Compiler](https://norasandler.com/2017/11/29/Write-a-Compiler.html) blog post series by Nora Sandler, which is also
available as a [book](https://norasandler.com/book/).
- The compiler reads a C source file and compiles it to x86-64 assembly ([AT&T syntax](https://en.wikipedia.org/wiki/X86_assembly_language#Syntax)).
- In debug mode, the assembly is assembled and linked using gcc, the resulting executable is run and the return value is printed to the console.

---

### Build and Run:
The compiler is written in Rust and can be built using Cargo:
```bash
# Release build: Compiles to x86-64 assembly.
cargo build --release
cargo run --release --bin compiler <source_file.c>

# Debug build: Additionally assembles and links the code using gcc, and then runs the resulting executable.
cargo build
cargo run --bin compiler <source_file.c>
```

### Testing:
A set of end-to-end tests is included which verify the return value of the compiled code:
```
cargo test
```
The test sets are mostly adopted from [Nora Sandler's provided tests](https://github.com/nlsandler/write_a_c_compiler).

## Main features:
- Reads a C source file and compiles it to x86-64 assembly (AT&T).
- Supports a (very) small subset of C, end goal is to support all of the following:
  - Arithmetic expressions.
  - Variable declarations and assignments.
  - Control flow (if/else, loops).
  - Function definitions and calls.
  - (pointers and arrays).
  - (calls to std library functions, e.g. printf).
- Custom lexer, recursive descent parser, and code generator.

---

## Future topics to cover:
- Use LLVM for code generation instead of generating assembly directly.
- Support GPU acceleration using CUDA.