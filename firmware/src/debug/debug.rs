use core::fmt::Write;

use crate::drivers::clickwheel::Event;
use crate::graphics::color::{BLACK, GRAY, GREEN, RED, WHITE};
use crate::graphics::framebuffer::Framebuffer;
use crate::graphics::text::Text;
use crate::platform::display::{LCD_HEIGHT, LCD_WIDTH};

const TOP: usize = 24;
const MARGIN: usize = 20;
// label line above each bar
const LABEL: usize = 10;

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

// one labelled time in us per bar, like compare() returns
pub fn draw_timings<const N: usize>(fb: &mut Framebuffer, title: &str, timings: [(&str, u32, u16); N]) {
    assert!(N != 0 && N <= 8);
    fb.clear();

    let longest = timings.iter().map(|&(_, time, _)| time).max().unwrap_or(1).max(1);
    let slot = (LCD_HEIGHT - TOP) / N;
    let height = (slot - LABEL - 4).min(16);

    // All bars share one scale: longer means slower.
    fb.fill_rect(MARGIN - 1, TOP, 1, LCD_HEIGHT - TOP, WHITE);
    fb.fill_rect(LCD_WIDTH - MARGIN, TOP, 1, LCD_HEIGHT - TOP, GRAY);

    for (index, &(_, time, color)) in timings.iter().enumerate() {
        let width = (u64::from(time) * (LCD_WIDTH - 2 * MARGIN) as u64 / u64::from(longest)) as usize;
        fb.fill_rect(MARGIN, TOP + index * slot + LABEL, width, height, color);
    }

    let mut text = Text::new(fb);
    text.move_to(MARGIN, 8);
    let _ = write!(text, "{title}");
    for (index, (label, time, _)) in timings.into_iter().enumerate() {
        text.move_to(MARGIN, TOP + index * slot);
        let _ = write!(text, "{label:<22}{time:>7} us");
    }
}

pub fn draw_events(fb: &mut Framebuffer, events: &[Option<Event>], dropped: u32) {
    fb.clear();
    let mut text = Text::new(fb);
    text.set_scale(2);
    text.move_to(8, 8);

    let _ = writeln!(text, "dropped {dropped}\n");
    for event in events.iter().flatten() {
        let _ = writeln!(text, "{event:?}");
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
