ENTRY(_start)

MEMORY
{
    /* SDRAM is physically at 0x10000000, start.s remaps it to 0 */
    DRAM : ORIGIN = 0x00000000, LENGTH = 32M
    IRAM : ORIGIN = 0x40000000, LENGTH = 48K
}

SECTIONS
{
    .text : {
        KEEP(*(.text.init))
        *(.text*)
    } > DRAM

    .ARM.exidx : {
        *(.ARM.exidx*)
    } > DRAM

    .rodata : {
        *(.rodata*)
    } > DRAM

    .data : {
        *(.data*)
        . = ALIGN(4);
    } > DRAM

    /* Stored in the DRAM image, copied to its execution address at startup. */
    .iram : {
        _iramstart = .;
        KEEP(*(.icode*))
        . = ALIGN(4);
        _iramend = .;
    } > IRAM AT> DRAM
    _iramcopy = LOADADDR(.iram);
    ASSERT(_iramcopy + SIZEOF(.iram) <= 8M, "linux.bin exceeds the 8MB bootloader limit")

    .bss (_iramcopy + SIZEOF(.iram)) (NOLOAD) : {
        /* zeroing will write 4 byte words */
        . = ALIGN(4);
        _bssbegin = .;
        *(.bss*)
        *(COMMON)
        . = ALIGN(4);
        _bssend = .;
    } > DRAM

    .stack (NOLOAD) : {
        . = ALIGN(8);
        _stackbegin = .;
        /* stacks grow down, so each symbol is the top of one. the system
           stack is lowest: an overflow hits .bss, not the exception stacks */
        . += 0x2000;
        _systemstack = .;
        . += 0x400;
        _irqstack = .;
        . += 0x200;
        _fiqstack = .;
        . += 0x800;
        _exceptionstack = .;
        _stackend = .;
    } > DRAM
}
