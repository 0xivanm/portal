#![no_std]
#![no_main]

mod drivers;
mod graphics;
mod platform;
mod debug;
mod sync;

use fat::Volume;

use crate::debug::debug::draw_checks;
use crate::drivers::{ata, backlight, lcd};
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

    // temporary: the mounted volume against what Windows reports for E:
    let volume = Volume::mount(ata::init().unwrap()).unwrap();
    let fb = framebuffer::take().unwrap();
    draw_checks(fb, [
        ("start", volume.start() as u32, volume.start() == 192_780),
        ("cluster", volume.cluster_size(), volume.cluster_size() == 16_384),
        ("clusters", volume.clusters(), volume.clusters() == 7_813_515),
        ("free", volume.free_clusters().unwrap_or(0), volume.free_clusters() == Some(7_133_183)),
        ("volume id", volume.volume_id(), volume.volume_id() == 0xCDA5_911C),
    ]);
    lcd.update(fb.pixels());

    loop {
        core::hint::spin_loop();
    }
}
