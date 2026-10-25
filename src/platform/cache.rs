// Runs from IRAM so switching or discarding the cache never touches the code doing it
use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

use crate::platform::registers::{CACHE_CTL, CACHE_MASK, CACHE_OPERATION, CACHE_PRIORITY};

const ENABLE: u32 = 0x0001;
const RUN: u32 = 0x0002;
const INIT: u32 = 0x0004;
const BUSY: u32 = 0x8000;
const OP_FLUSH: u32 = 0x0002;

const SIZE: usize = 0x2000;
const LINE: usize = 16;
// one status word per line, for the CPU core
const STATUS_BASE: usize = 0xF000_4000;
const LINE_VALID: u32 = 1 << 23;
// the highest line address, which nothing ever uses
const LINE_UNUSED: u32 = (1 << 21) - 1;

#[unsafe(link_section = ".icode.cache")]
#[inline(never)]
pub fn enable() {
    unsafe {
        CACHE_CTL.modify(|value| (value & !(ENABLE | RUN)) | INIT);
        // rockbox sets this for the CPU core too, without knowing what it does
        CACHE_PRIORITY.modify(|value| value | 0x10);
        // cache 0x00000000-0x03ffffff and 0x20000000-0x23ffffff
        CACHE_MASK.write(0x0000_1C00);
        CACHE_OPERATION.write(0x0000_0FC0);
        CACHE_CTL.modify(|value| value | INIT | ENABLE | RUN);
        nops();

        // make every line valid by reading one cache size of cached memory
        let mut address = SIZE;
        while address < 2 * SIZE {
            read_volatile(address as *const u8);
            address += LINE;
        }
    }
}

#[unsafe(link_section = ".icode.cache")]
#[inline(never)]
pub fn disable() {
    discard();
    unsafe { CACHE_CTL.modify(|value| value & !(ENABLE | RUN)) };
}

#[inline(always)]
pub fn is_enabled() -> bool {
    unsafe { CACHE_CTL.read() & ENABLE != 0 }
}

// writes dirty lines back to SDRAM
#[unsafe(link_section = ".icode.cache")]
#[inline(never)]
pub fn flush() {
    if !is_enabled() {
        return;
    }
    unsafe {
        CACHE_OPERATION.modify(|value| value | OP_FLUSH);
        while CACHE_CTL.read() & BUSY != 0 {
            core::hint::spin_loop();
        }
        nops();
    }
}

// flushes, then forgets every line. a plain invalidate can corrupt memory on
// this chip, so rockbox instead marks each line valid and clean for an unused address
#[unsafe(link_section = ".icode.cache")]
#[inline(never)]
pub fn discard() {
    if !is_enabled() {
        return;
    }
    flush();
    let mut address = STATUS_BASE;
    while address < STATUS_BASE + SIZE {
        unsafe { write_volatile(address as *mut u32, LINE_VALID | LINE_UNUSED) };
        address += LINE;
    }
}

#[inline(always)]
fn nops() {
    unsafe { asm!("nop", "nop", "nop", "nop", options(nomem, nostack, preserves_flags)) };
}
