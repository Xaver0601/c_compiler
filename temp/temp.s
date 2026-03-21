.globl main
main:
movl $5, %eax
negl %eax
movl %eax, %eax
ret
.section .note.GNU-stack,"",@progbits
