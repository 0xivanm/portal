#![no_std]
#![no_main]

mod drivers;
mod graphics;
mod platform;
mod debug;

use crate::debug::debug::draw_charset;
use crate::drivers::{backlight, lcd};
use crate::graphics::framebuffer;
use crate::platform::{cache, clock, irq};

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    clock::set_frequency(clock::CpuFrequency::Mhz30);
    cache::enable();
    irq::mask_all();
    irq::enable();
    let mut lcd = lcd::init();
    backlight::on();

    let fb = framebuffer::take().unwrap();
    draw_charset(fb);
    lcd.update(fb.pixels());

    loop {
        core::hint::spin_loop();
    }
}
