use core::arch::global_asm;
use core::cell::UnsafeCell;

use crate::platform::display::{LCD_HEIGHT, LCD_WIDTH};

global_asm!(include_str!("copy.s"), options(raw));

pub const FB_WIDTH: usize = LCD_WIDTH;
pub const FB_HEIGHT: usize = LCD_HEIGHT;

#[repr(align(4))]
pub struct Framebuffer {
    pixels: [u16; FB_WIDTH * FB_HEIGHT],
}

struct Storage(UnsafeCell<Framebuffer>);

// Only take() accesses this storage, once during startup.
unsafe impl Sync for Storage {}

static FRAMEBUFFER: Storage = Storage(UnsafeCell::new(Framebuffer {
    pixels: [0; FB_WIDTH * FB_HEIGHT],
}));

pub unsafe fn take() -> &'static mut Framebuffer {
    unsafe { &mut *FRAMEBUFFER.0.get() }
}

unsafe extern "C" {
    fn copy_framebuffer_asm_raw(destination: *mut u8, source: *const u8, bytes: usize);
}

impl Framebuffer {
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    #[inline(never)]
    pub fn fill(&mut self, image: &[u8]) {
        assert_eq!(image.len(), FB_WIDTH * FB_HEIGHT * 2);
        unsafe {
            copy_framebuffer_asm_raw(self.pixels.as_mut_ptr().cast::<u8>(), image.as_ptr(), image.len());
        }
    }

    pub fn pixels(&self) -> &[u16; FB_WIDTH * FB_HEIGHT] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u16; FB_WIDTH * FB_HEIGHT] {
        &mut self.pixels
    }
}
