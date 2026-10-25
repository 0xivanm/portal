#![no_std]
#![no_main]

mod drivers;
mod graphics;
mod platform;
mod debug;
mod sync;

use crate::debug::debug::draw_events;
use crate::drivers::{backlight, clickwheel, lcd};
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
    let mut wheel = clickwheel::init();

    let fb = framebuffer::take().unwrap();
    let mut events = [None; 12];
    let mut changed = true;
    loop {
        wheel.poll(|event| {
            events.rotate_right(1);
            events[0] = Some(event);
            changed = true;
        });
        if changed {
            changed = false;
            draw_events(fb, &events, wheel.dropped());
            lcd.update(fb.pixels());
        }
    }
}
