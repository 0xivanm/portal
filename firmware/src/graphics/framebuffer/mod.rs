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

// Only take() and steal() access this storage
unsafe impl Sync for Storage {}

static FRAMEBUFFER: Storage = Storage(UnsafeCell::new(Framebuffer {
    pixels: [0; FB_WIDTH * FB_HEIGHT],
}));

static mut TAKEN: bool = false;

// hands out the framebuffer once. not interrupt safe, so take it at startup
pub fn take() -> Option<&'static mut Framebuffer> {
    if unsafe { TAKEN } {
        return None;
    }
    unsafe { TAKEN = true };
    Some(unsafe { steal() })
}

/// # Safety
/// Aliases the owner's reference: only for code that never returns to it, like the crash screen.
pub unsafe fn steal() -> &'static mut Framebuffer {
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
    pub fn fill(&mut self, image: &[u16]) {
        assert_eq!(image.len(), FB_WIDTH * FB_HEIGHT);
        // the asm copies whole words
        assert_eq!(image.as_ptr() as usize & 3, 0);
        unsafe {
            copy_framebuffer_asm_raw(self.pixels.as_mut_ptr().cast::<u8>(), image.as_ptr().cast::<u8>(), image.len() * 2);
        }
    }

    // the visible part of a horizontal strip, None if it is entirely off screen.
    // all clipping happens here, so no drawing can index out of bounds
    pub fn span_mut(&mut self, x: usize, y: usize, width: usize) -> Option<&mut [u16]> {
        if x >= FB_WIDTH || y >= FB_HEIGHT {
            return None;
        }
        let start = y * FB_WIDTH + x;
        Some(&mut self.pixels[start..start + width.min(FB_WIDTH - x)])
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u16) {
        for row in y..y.saturating_add(height) {
            match self.span_mut(x, row, width) {
                Some(span) => span.fill(color),
                None => break,
            }
        }
    }

    pub fn pixels(&self) -> &[u16; FB_WIDTH * FB_HEIGHT] {
        &self.pixels
    }
}
