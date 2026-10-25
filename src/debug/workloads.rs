use core::slice;

use crate::drivers::lcd;
use crate::graphics::framebuffer::Framebuffer;
use crate::platform::registers::BCM_DATA32;

#[repr(align(4))]
struct Image([u8; include_bytes!("../../assets/image.raw").len()]);

static IMAGE: Image = Image(*include_bytes!("../../assets/image.raw"));

// one full frame of RGB565, little endian
pub fn image() -> &'static [u16] {
    // the wrapper aligns the bytes for u16 and word access
    unsafe { slice::from_raw_parts(IMAGE.0.as_ptr().cast(), IMAGE.0.len() / 2) }
}

// Generate identical Rust bodies, differing only in code placement.
macro_rules! rust_transfer {
    ($name:ident, $section:literal) => {
        #[unsafe(link_section = $section)]
        #[inline(never)]
        pub fn $name(fb: &mut Framebuffer) {
            for pair in fb.pixels().chunks_exact(2) {
                let value = u32::from(pair[0]) | (u32::from(pair[1]) << 16);
                unsafe { BCM_DATA32.write(value) };
            }
        }
    };
}

rust_transfer!(lcd_transfer_sdram, ".text.lcd_transfer");
rust_transfer!(lcd_transfer_iram, ".icode.lcd_transfer");

pub fn lcd_transfer_asm(fb: &mut Framebuffer) {
    lcd::lcd_write_data(fb.pixels());
}

pub fn copy_rust(fb: &mut Framebuffer) {
    fb.pixels_mut().copy_from_slice(image());
}

pub fn copy_asm(fb: &mut Framebuffer) {
    fb.fill(image());
}
