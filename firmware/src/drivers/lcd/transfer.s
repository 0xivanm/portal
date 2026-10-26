@ Adapted from Rockbox firmware/target/arm/ipod/video/lcd-as-video.S.

.syntax unified
.arm
.section .icode.asm, "ax", %progbits
.balign 4
.global lcd_write_data_asm_raw
.type lcd_write_data_asm_raw, %function

@ r0: word-aligned pixels, r1: even pixel count
lcd_write_data_asm_raw:
    stmdb sp!, {r4, lr}
    mov lr, #0x30000000

    subs r1, r1, #16
1:
    ldmiage r0!, {r2-r3}
    stmiage lr, {r2-r3}
    ldmiage r0!, {r2-r3}
    stmiage lr, {r2-r3}
    ldmiage r0!, {r2-r3}
    stmiage lr, {r2-r3}
    ldmiage r0!, {r2-r3}
    stmiage lr, {r2-r3}
    subsge r1, r1, #16
    bge 1b

    tst r1, #8
    ldmiane r0!, {r2-r4, r12}
    stmiane lr, {r2-r4, r12}
    tst r1, #4
    ldmiane r0!, {r2-r3}
    stmiane lr, {r2-r3}
    tst r1, #2
    ldrne r3, [r0], #4
    strne r3, [lr]

    ldmia sp!, {r4, lr}
    bx lr
    
.size lcd_write_data_asm_raw, . - lcd_write_data_asm_raw
