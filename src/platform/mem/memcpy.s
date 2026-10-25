@ Ported from Rockbox firmware/asm/arm/memcpy.S, originally glibc (Nicolas Pitre).
@ GPL v2 or later. Copies 8 words per iteration; when source and destination are
@ misaligned relative to each other it shifts bytes between registers instead of
@ falling back to byte copies.

.syntax unified
.arm

@ copies words from a source that is 1, 2 or 3 bytes past a word boundary
.macro forward_copy_shift pull push
    subs r2, r2, #28
    blt 14f

11: stmfd sp!, {r5-r9}

12:
13: ldmia r1!, {r4, r5, r6, r7}
    mov r3, lr, lsr #\pull
    subs r2, r2, #32
    ldmia r1!, {r8, r9, ip, lr}
    orr r3, r3, r4, lsl #\push
    mov r4, r4, lsr #\pull
    orr r4, r4, r5, lsl #\push
    mov r5, r5, lsr #\pull
    orr r5, r5, r6, lsl #\push
    mov r6, r6, lsr #\pull
    orr r6, r6, r7, lsl #\push
    mov r7, r7, lsr #\pull
    orr r7, r7, r8, lsl #\push
    mov r8, r8, lsr #\pull
    orr r8, r8, r9, lsl #\push
    mov r9, r9, lsr #\pull
    orr r9, r9, ip, lsl #\push
    mov ip, ip, lsr #\pull
    orr ip, ip, lr, lsl #\push
    stmia r0!, {r3, r4, r5, r6, r7, r8, r9, ip}
    bge 12b

    ldmfd sp!, {r5-r9}

14: ands ip, r2, #28
    beq 16f

15: mov r3, lr, lsr #\pull
    ldr lr, [r1], #4
    subs ip, ip, #4
    orr r3, r3, lr, lsl #\push
    str r3, [r0], #4
    bgt 15b

16: sub r1, r1, #(\push / 8)
    b 8b
.endm

@ void *memcpy(void *dest, const void *src, size_t n)
@ LLVM calls the ARM EABI variants instead, which take the same arguments, so
@ they are just more names for the same code
.section .text.memcpy, "ax", %progbits
.balign 4
.global memcpy
.global __aeabi_memcpy
.global __aeabi_memcpy4
.global __aeabi_memcpy8
.type memcpy, %function
.type __aeabi_memcpy, %function
.type __aeabi_memcpy4, %function
.type __aeabi_memcpy8, %function
memcpy:
__aeabi_memcpy:
__aeabi_memcpy4:
__aeabi_memcpy8:
    stmfd sp!, {r0, r4, lr}

0:  subs r2, r2, #4
    blt 8f
    ands ip, r0, #3
    bne 9f
    ands ip, r1, #3
    bne 10f

1:  subs r2, r2, #28
    stmfd sp!, {r5-r8}
    blt 5f

2:
3:
4:  ldmia r1!, {r3, r4, r5, r6, r7, r8, ip, lr}
    subs r2, r2, #32
    stmia r0!, {r3, r4, r5, r6, r7, r8, ip, lr}
    bge 3b

    @ jumps into the ldr and str runs below to copy the remaining 0-7 words
5:  ands ip, r2, #28
    rsb ip, ip, #32
    addne pc, pc, ip
    b 7f
6:  nop
    ldr r3, [r1], #4
    ldr r4, [r1], #4
    ldr r5, [r1], #4
    ldr r6, [r1], #4
    ldr r7, [r1], #4
    ldr r8, [r1], #4
    ldr lr, [r1], #4

    add pc, pc, ip
    nop
    nop
    str r3, [r0], #4
    str r4, [r0], #4
    str r5, [r0], #4
    str r6, [r0], #4
    str r7, [r0], #4
    str r8, [r0], #4
    str lr, [r0], #4

7:  ldmfd sp!, {r5-r8}

    @ the last 0-3 bytes
8:  movs r2, r2, lsl #31
    ldrbne r3, [r1], #1
    ldrbcs r4, [r1], #1
    ldrbcs ip, [r1]
    strbne r3, [r0], #1
    strbcs r4, [r0], #1
    strbcs ip, [r0]

    ldmfd sp!, {r0, r4, lr}
    bx lr

    @ destination not word aligned: copy 1-3 bytes to align it
9:  rsb ip, ip, #4
    cmp ip, #2
    ldrbgt r3, [r1], #1
    ldrbge r4, [r1], #1
    ldrb lr, [r1], #1
    strbgt r3, [r0], #1
    strbge r4, [r0], #1
    subs r2, r2, ip
    strb lr, [r0], #1
    blt 8b
    ands ip, r1, #3
    beq 1b

    @ source still misaligned: shift bytes between words
10: bic r1, r1, #3
    cmp ip, #2
    ldr lr, [r1], #4
    beq 17f
    bgt 18f

    forward_copy_shift pull=8 push=24
17: forward_copy_shift pull=16 push=16
18: forward_copy_shift pull=24 push=8
.size memcpy, . - memcpy
