#![no_std]
#![no_main]

mod drivers;
mod graphics;
mod platform;
mod debug;

use crate::debug::benchmark::compare;
use crate::debug::debug::{draw_charset, draw_timings};
use crate::debug::workloads::{copy_asm, copy_rust, image, lcd_transfer_asm, lcd_transfer_iram, lcd_transfer_sdram};
use crate::drivers::{backlight, lcd};
use crate::graphics::color::{BLUE, GREEN, RED};
use crate::graphics::framebuffer::{self, Framebuffer};
use crate::platform::{cache, clock, timer};

const ROUNDS: u32 = 12;

// one block per cache state, so the cache stays warm while it is on
fn measure<const N: usize>(
    fb: &mut Framebuffer,
    prepare: impl FnMut() + Copy,
    tests: [fn(&mut Framebuffer); N],
    verify: impl FnMut(&Framebuffer) + Copy,
) -> ([u32; N], [u32; N]) {
    cache::disable();
    let off = compare(fb, ROUNDS, prepare, tests, verify);
    cache::enable();
    let on = compare(fb, ROUNDS, prepare, tests, verify);
    (off, on)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    clock::set_frequency(clock::CpuFrequency::Mhz30);
    let mut lcd = lcd::init();
    backlight::on();

    let fb = framebuffer::take().unwrap();
    let (transfer_off, transfer_on) =
        measure(fb, lcd::prepare_transfer, [lcd_transfer_sdram, lcd_transfer_iram, lcd_transfer_asm], |_| {});
    let (copy_off, copy_on) =
        measure(fb, || {}, [copy_rust, copy_asm], |fb| assert!(fb.pixels()[..] == image()[..]));
    let (render_off, render_on) = measure(fb, || {}, [draw_charset], |_| {});

    loop {
        draw_timings(fb, "lcd transfer", [
            ("rust sdram  cache off", transfer_off[0], RED),
            ("rust sdram  cache on", transfer_on[0], RED),
            ("rust iram   cache off", transfer_off[1], GREEN),
            ("rust iram   cache on", transfer_on[1], GREEN),
            ("asm iram    cache off", transfer_off[2], BLUE),
            ("asm iram    cache on", transfer_on[2], BLUE),
        ]);
        lcd.update(fb.pixels());
        timer::delay_ms(5000);

        draw_timings(fb, "image copy", [
            ("rust sdram  cache off", copy_off[0], RED),
            ("rust sdram  cache on", copy_on[0], RED),
            ("asm iram    cache off", copy_off[1], BLUE),
            ("asm iram    cache on", copy_on[1], BLUE),
        ]);
        lcd.update(fb.pixels());
        timer::delay_ms(5000);

        draw_timings(fb, "charset render", [
            ("rust sdram  cache off", render_off[0], RED),
            ("rust sdram  cache on", render_on[0], RED),
        ]);
        lcd.update(fb.pixels());
        timer::delay_ms(5000);
    }
}
