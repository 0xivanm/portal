@ Ported from Rockbox firmware/asm/arm/memset.S, based on Linux 2.6.15.3
@ arch/arm/lib/memset.S (Russell King). GPL v2 or later. Writes backwards
@ from the end, 64 bytes per iteration.

.syntax unified
.arm

.section .text.memset, "ax", %progbits
.balign 4

@ LLVM calls the ARM EABI variants instead, which take the count before the
@ value: (dest, n, c) and (dest, n) for memclr. These reorder the arguments
.global __aeabi_memset
.global __aeabi_memset4
.global __aeabi_memset8
.type __aeabi_memset, %function
.type __aeabi_memset4, %function
.type __aeabi_memset8, %function
__aeabi_memset:
__aeabi_memset4:
__aeabi_memset8:
    mov r3, r1
    mov r1, r2
    mov r2, r3
    b memset

.global __aeabi_memclr
.global __aeabi_memclr4
.global __aeabi_memclr8
.type __aeabi_memclr, %function
.type __aeabi_memclr4, %function
.type __aeabi_memclr8, %function
__aeabi_memclr:
__aeabi_memclr4:
__aeabi_memclr8:
    mov r2, r1
    mov r1, #0
    b memset

@ void *memset(void *dest, int c, size_t n)
    @ aligns the end pointer, entered from below
1:  cmp r2, #4
    blt 5f
    cmp r3, #2
    strbgt r1, [r0, #-1]!
    strbge r1, [r0, #-1]!
    strb r1, [r0, #-1]!
    sub r2, r2, r3
    b 2f

.global memset
.type memset, %function
memset:
    add r0, r0, r2
    ands r3, r0, #3
    bne 1b
2:
    @ the end is word aligned: spread the byte over a whole word
    and r1, r1, #255
    orr r1, r1, r1, lsl #8
    orr r1, r1, r1, lsl #16
    mov r3, r1
    cmp r2, #16
    blt 5f

    str lr, [sp, #-4]!
    mov ip, r1
    mov lr, r1

3:  subs r2, r2, #64
    stmdbge r0!, {r1, r3, ip, lr}
    stmdbge r0!, {r1, r3, ip, lr}
    stmdbge r0!, {r1, r3, ip, lr}
    stmdbge r0!, {r1, r3, ip, lr}
    bgt 3b
    ldreq lr, [sp], #4
    bxeq lr

    @ fewer than 64 bytes left; only the low bits of the count matter now
    tst r2, #32
    stmdbne r0!, {r1, r3, ip, lr}
    stmdbne r0!, {r1, r3, ip, lr}
    tst r2, #16
    stmdbne r0!, {r1, r3, ip, lr}
    ldr lr, [sp], #4

5:  tst r2, #8
    stmdbne r0!, {r1, r3}
    tst r2, #4
    strne r1, [r0, #-4]!

    @ fewer than 4 bytes left
6:  tst r2, #2
    strbne r1, [r0, #-1]!
    strbne r1, [r0, #-1]!
    tst r2, #1
    strbne r1, [r0, #-1]!
    bx lr
.size memset, . - memset
