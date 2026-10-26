use core::arch::asm;

use crate::drivers::clickwheel;
use crate::platform::registers::{
    COP_HI_INT_DIS, COP_INT_DIS, CPU_HI_INT_DIS, CPU_HI_INT_EN, CPU_HI_INT_STAT, CPU_INT_DIS, CPU_INT_EN,
    CPU_INT_STAT, HI_INT_FORCED_CLR, INT_FORCED_CLR, Register32,
};

// interrupt enables of GPIO ports A-L, four ports per block
const GPIO_INT_EN: [usize; 3] = [0x6000_D050, 0x6000_D0D0, 0x6000_D150];

// the bootloader may have left sources enabled: disable_all_interrupts
pub fn mask_all() {
    unsafe {
        COP_HI_INT_DIS.write(u32::MAX);
        CPU_HI_INT_DIS.write(u32::MAX);
        HI_INT_FORCED_CLR.write(u32::MAX);
        COP_INT_DIS.write(u32::MAX);
        CPU_INT_DIS.write(u32::MAX);
        INT_FORCED_CLR.write(u32::MAX);
        for block in GPIO_INT_EN {
            for port in 0..4 {
                Register32::new(block + port * 4).write(0);
            }
        }
    }
}

// sources 32-63 sit in a second level, which reaches the first through this one bit
const HI: u32 = 1 << 30;

// the controller has separate enable and disable registers: only the written bits change
pub fn enable_source(mask: u32) {
    unsafe { CPU_INT_EN.write(mask) };
}

pub fn enable_hi_source(mask: u32) {
    unsafe { CPU_HI_INT_EN.write(mask) };
    enable_source(HI);
}

pub fn disable_hi_source(mask: u32) {
    unsafe { CPU_HI_INT_DIS.write(mask) };
}

// clears the I bit in the cpsr
pub fn enable() {
    unsafe { asm!("mrs {0}, cpsr", "bic {0}, {0}, #0x80", "msr cpsr_c, {0}", out(reg) _) };
}

// handles every pending source in one entry
#[unsafe(no_mangle)]
extern "C" fn irq_dispatch() {
    let mut pending = unsafe { CPU_INT_STAT.read() };
    let mut pending_hi = 0;
    if pending & HI != 0 {
        pending &= !HI;
        pending_hi = unsafe { CPU_HI_INT_STAT.read() };
    }

    if pending_hi & clickwheel::IRQ != 0 {
        clickwheel::interrupt();
        pending_hi &= !clickwheel::IRQ;
    }

    // a source without a handler would fire forever
    if pending != 0 || pending_hi != 0 {
        panic!("unexpected irq {pending:08X} hi {pending_hi:08X}");
    }
}
