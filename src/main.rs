#![no_std]
#![no_main]

mod drivers;
mod graphics;
mod platform;
mod debug;

use core::ptr::{read_volatile, write_volatile};

use crate::debug::debug::{draw_charset, draw_checks};
use crate::drivers::{backlight, lcd};
use crate::graphics::framebuffer;
use crate::platform::{clock, timer};
use crate::platform::registers::{MMAP0_LOGICAL, MMAP0_PHYSICAL};

const UNCACHED_BASE: usize = 0x1000_0000;

static mut PROBE: u32 = 0;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    clock::set_frequency(clock::CpuFrequency::Mhz30);
    let mut lcd = lcd::init();
    backlight::on();

    let logical = &raw mut PROBE;
    let physical = (logical as usize | UNCACHED_BASE) as *mut u32;

    // both addresses must reach the same word
    let (forward, backward) = unsafe {
        write_volatile(logical, 0x1234_5678);
        let forward = read_volatile(physical);
        write_volatile(physical, 0x8765_4321);
        (forward, read_volatile(logical))
    };
    let (mmap_logical, mmap_physical) = unsafe { (MMAP0_LOGICAL.read(), MMAP0_PHYSICAL.read()) };

    let entry = rust_main as *const () as usize;
    let rows = [
        ("rust_main", entry as u32, entry < UNCACHED_BASE),
        ("probe", logical as u32, (logical as usize) < UNCACHED_BASE),
        ("mmap0 log", mmap_logical, mmap_logical == 0x0000_3E00),
        ("mmap0 phys", mmap_physical, mmap_physical == 0x1000_0F84),
        ("alias fwd", forward, forward == 0x1234_5678),
        ("alias back", backward, backward == 0x8765_4321),
    ];

    let fb = unsafe { framebuffer::take() };
    loop {
        draw_checks(fb, rows);
        lcd.update(fb.pixels());
        timer::delay_ms(3000);

        draw_charset(fb);
        lcd.update(fb.pixels());
        timer::delay_ms(3000);
    }
}
