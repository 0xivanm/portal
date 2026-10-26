@ adaptded from rockbox

.syntax unified
.arm
.section .icode.copy, "ax", %progbits
.balign 4
.global copy_framebuffer_asm_raw
.type copy_framebuffer_asm_raw, %function

@ r0: aligned destination, r1: aligned source, r2: bytes (multiple of 32)
copy_framebuffer_asm_raw:
    stmdb sp!, {r4-r8, lr}
    cmp r2, #0
    beq 2f
1:
    ldmia r1!, {r3-r8, r12, lr}
    subs r2, r2, #32
    stmia r0!, {r3-r8, r12, lr}
    bne 1b
2:
    ldmia sp!, {r4-r8, lr}
    bx lr
.size copy_framebuffer_asm_raw, . - copy_framebuffer_asm_raw
