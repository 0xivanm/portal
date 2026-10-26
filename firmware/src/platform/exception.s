.syntax unified
.arm
.section .text.exception, "ax", %progbits

@ saves the interrupted registers as a frame on this mode's stack and calls
@ exception(kind, frame) in rust. frame: r0-r12, sp, lr, pc, psr (17 words,
@ 72 bytes reserved to keep sp 8-byte aligned)
.macro handler name, offset, kind
.global \name
\name:
    @ lr points past the faulting instruction
    sub lr, lr, #\offset
    sub sp, sp, #72
    stmia sp, {r0-r12}
    @ ^ stores the interrupted system mode's sp and lr, not this mode's
    add r0, sp, #52
    stmia r0, {sp, lr}^
    mrs r1, spsr
    str lr, [sp, #60]
    str r1, [sp, #64]
    mov r0, #\kind
    mov r1, sp
    b exception
.endm

handler undefined, 4, 0
handler swi, 4, 1
handler prefetch_abort, 4, 2
handler data_abort, 8, 3
handler fiq, 4, 5

@ returns to the interrupted code, so only the registers a call may overwrite
@ are saved. ^ with pc in the list also restores the cpsr from the spsr
.global irq
irq:
    sub lr, lr, #4
    stmdb sp!, {r0-r3, r12, lr}
    bl irq_dispatch
    ldmia sp!, {r0-r3, r12, pc}^
