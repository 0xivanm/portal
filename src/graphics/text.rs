use core::fmt;

use crate::graphics::color::{BLACK, WHITE};
use crate::graphics::font;
use crate::graphics::framebuffer::{FB_HEIGHT, FB_WIDTH, Framebuffer};

pub struct Text<'a> {
    fb: &'a mut Framebuffer,
    x: usize,
    y: usize,
    // where lines restart after a newline
    left: usize,
    scale: usize,
    fg: u16,
    bg: u16,
}

impl<'a> Text<'a> {
    pub fn new(fb: &'a mut Framebuffer) -> Self {
        Self { fb, x: 0, y: 0, left: 0, scale: 1, fg: WHITE, bg: BLACK }
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.left = x;
    }

    pub fn set_scale(&mut self, scale: usize) {
        self.scale = scale.max(1);
    }

    pub fn set_color(&mut self, fg: u16, bg: u16) {
        self.fg = fg;
        self.bg = bg;
    }

    pub fn newline(&mut self) {
        self.x = self.left;
        self.y += font::HEIGHT * self.scale;
    }

    // characters that don't fit are dropped, so drawing can never panic
    fn draw_char(&mut self, c: char) {
        let width = font::WIDTH * self.scale;
        if self.x + width > FB_WIDTH {
            self.newline();
        }
        if self.x + width > FB_WIDTH || self.y + font::HEIGHT * self.scale > FB_HEIGHT {
            return;
        }

        let pixels = self.fb.pixels_mut();
        for (row, bits) in font::glyph(c).iter().enumerate() {
            for column in 0..font::WIDTH {
                let color = if bits & (0x80 >> column) != 0 { self.fg } else { self.bg };
                let x = self.x + column * self.scale;
                for y in self.y + row * self.scale..self.y + (row + 1) * self.scale {
                    pixels[y * FB_WIDTH + x..][..self.scale].fill(color);
                }
            }
        }
        self.x += width;
    }
}

impl fmt::Write for Text<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => self.newline(),
                c => self.draw_char(c),
            }
        }
        Ok(())
    }
}
