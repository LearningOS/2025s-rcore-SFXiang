    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top // 栈顶
    call rust_main

    .section .bss.stack
    .globl boot_stack_lower_bound // 栈底
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top: