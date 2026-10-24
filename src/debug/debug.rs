use core::fmt::Write;

use crate::graphics::color::{BLACK, GREEN, RED};
use crate::graphics::framebuffer::Framebuffer;
use crate::graphics::text::Text;

// one labelled value per row: green if the check passed, red if not
pub fn draw_checks<const N: usize>(fb: &mut Framebuffer, checks: [(&str, u32, bool); N]) {
    fb.clear();
    let mut text = Text::new(fb);
    text.set_scale(2);
    text.move_to(16, 16);

    for (label, value, ok) in checks {
        text.set_color(if ok { GREEN } else { RED }, BLACK);
        let _ = writeln!(text, "{label:<12}{value:08X}");
    }
}

// every printable character at scales 1 to 3
pub fn draw_charset(fb: &mut Framebuffer) {
    fb.clear();
    let mut text = Text::new(fb);

    for scale in 1..=3 {
        text.set_scale(scale);
        for c in ' '..='~' {
            let _ = text.write_char(c);
        }
        text.newline();
    }
}
