use core::arch::asm;
use core::ptr::read_volatile;

pub fn undefined_instruction() -> ! {
    unsafe {
        asm!(
            // permanently undefined in the ARM instruction set
            ".word 0xe7f000f0",
            in("r0") 0x0BAD_C0DE,
            in("r1") 0x1111_1111,
            in("r2") 0x2222_2222,
            in("r3") 0x3333_3333,
            in("r4") 0x4444_4444,
            in("r5") 0x5555_5555,
            in("r12") 0xCCCC_CCCC_u32,
            options(noreturn),
        );
    }
}

pub fn software_interrupt() -> ! {
    unsafe { asm!("swi #0", options(noreturn)) }
}

// prefetch abort if nothing answers at the address. mov, not bx: an odd
// address would switch bx to thumb
pub fn jump_to(address: usize) -> ! {
    unsafe { asm!("mov pc, {0}", in(reg) address, options(noreturn)) }
}

// a data abort if nothing answers at the address, otherwise whatever it reads
pub fn load_from(address: usize) -> u32 {
    unsafe { read_volatile(address as *const u32) }
}
