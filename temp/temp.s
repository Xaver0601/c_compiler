.globl main
main:
movl $5, %eax
cmpl $0, %eax
movl $0, %eax
sete %al
ret
.section .note.GNU-stack,"",@progbits
