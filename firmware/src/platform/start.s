.section .text.init, "ax"
.global _start

.equ COP_STATUS, 0x60007004
.equ COP_CTRL, 0x60007004
.equ SLEEP, 0x80000000
.equ COPSLEEP, 0x80000000
.equ PROC_ID, 0x60000000
.equ CACHE_CTRL, 0x6000c000
.equ IRAM, 0x40000000
.equ MMAP0_LOGICAL, 0xf000f000
.equ MMAP0_PHYSICAL, 0xf000f004
@ logical base 0, 32MB window
.equ MMAP_MASK, 0x00003e00
@ read, write, data and code access
.equ MMAP_FLAGS, 0x00000f84
@ processor modes with IRQ and FIQ masked
.equ FIQ_MODE, 0xd1
.equ IRQ_MODE, 0xd2
.equ SVC_MODE, 0xd3
.equ ABT_MODE, 0xd7
.equ UND_MODE, 0xdb
.equ SYS_MODE, 0xdf

@ exception vectors, at address 0 after the remap. on an exception the cpu
@ jumps to a fixed address (0x00 reset, 0x04 undefined, ... 0x1c fiq), so
@ each entry must be exactly one 4-byte instruction, in this order
_start:
    b reset             @ relative branch: runs before the remap
    b undefined
    b swi
    b prefetch_abort
    b data_abort
    b .                 @ reserved
    b irq
    b fiq

reset:
    @ disable interrupts and enter system mode
    msr cpsr_c, #SYS_MODE

    @ which processor is this?
    ldr r0, =PROC_ID
    ldrb r0, [r0]
    cmp r0, #0x55
    beq cpu

cop:
    @ put cop to sleep
    ldr r2, =COP_CTRL
    mov r1, #SLEEP
    str r1, [r2]
    @ wait for cop to go to sleep
    nop
    nop
    nop

    @ if woken up, go back to sleep
    b cop

cpu:
    @ check if cop is asleep
    ldr r4, =COP_STATUS
3:
    ldr r3, [r4]
    tst r3, #COPSLEEP
    beq 3b

    @ the bootloader may have left the cache on
    ldr r2, =CACHE_CTRL
    mov r1, #0
    str r1, [r2]

    @ linked at 0 but still running at the physical address, so
    @ label addresses need the execution base added until remapped
    and r6, pc, #0xff000000
    ldr r3, =remap_start
    ldr r4, =remap_end
    orr r3, r3, r6
    orr r4, r4, r6

    @ remap from IRAM: the memory map can't change under our own fetches
    mov r2, #IRAM
1:
    ldr r5, [r3], #4
    str r5, [r2], #4
    cmp r3, r4
    blo 1b

    ldr r1, =MMAP0_LOGICAL
    ldr r2, =MMAP_MASK
    ldr r3, =MMAP0_PHYSICAL
    ldr r4, =MMAP_FLAGS
    orr r4, r4, r6
    mov pc, #IRAM

remap_start:
    str r2, [r1]
    str r4, [r3]
    @ continue at the link address
    ldr r1, remap_target
    bx r1
remap_target:
    .word remap_end
remap_end:

    @ zero out bss
    ldr r2, =_bssend
    ldr r1, =_bssbegin
    mov r0, #0
1:
    cmp r2, r1
    strhi r0, [r1], #4
    bhi 1b

    @ fill all stacks
    ldr r1, =_stackend
    ldr r2, =_stackbegin
    ldr r0, =0xDEADBEEF
1:
    cmp r1, r2
    strhi r0, [r2], #4
    bhi 1b

    @ each mode has its own sp, the fatal exceptions share one stack
    msr cpsr_c, #FIQ_MODE
    ldr sp, =_fiqstack
    msr cpsr_c, #IRQ_MODE
    ldr sp, =_irqstack
    msr cpsr_c, #SVC_MODE
    ldr sp, =_exceptionstack
    msr cpsr_c, #ABT_MODE
    ldr sp, =_exceptionstack
    msr cpsr_c, #UND_MODE
    ldr sp, =_exceptionstack
    msr cpsr_c, #SYS_MODE
    ldr sp, =_systemstack

    @ copy IRAM code after leaving the bootloader and sleeping the COP
    ldr r0, =_iramcopy
    ldr r1, =_iramstart
    ldr r2, =_iramend
1:
    cmp r1, r2
    ldrlo r3, [r0], #4
    strlo r3, [r1], #4
    blo 1b

    bl rust_main

hang:
    b hang
