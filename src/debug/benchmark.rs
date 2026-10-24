use crate::graphics::color::{GRAY, WHITE};
use crate::graphics::framebuffer::Framebuffer;
use crate::platform::display::{LCD_HEIGHT, LCD_WIDTH};
use crate::platform::timer;
use core::arch::asm;

pub fn compare<T: ?Sized, const N: usize>(
    input: &T,
    rounds: u32,
    mut prepare: impl FnMut(),
    tests: [fn(&T); N],
    mut verify: impl FnMut(&T),
) -> [u32; N] {
    assert!(rounds != 0 && N != 0);
    let mut totals = [0u64; N];

    for test in tests {
        prepare();
        test(core::hint::black_box(input));
        verify(input);
    }

    for round in 0..rounds {
        for offset in 0..N {
            let index = (round as usize + offset) % N;
            prepare();

            unsafe { asm!("", options(nostack, preserves_flags)) }
            let start = timer::micros();
            tests[index](core::hint::black_box(input));
            unsafe { asm!("", options(nostack, preserves_flags)) }
            let elapsed = timer::micros().wrapping_sub(start);

            verify(input);
            totals[index] += u64::from(elapsed);
        }
    }

    totals.map(|total| (total / u64::from(rounds)) as u32)
}

pub fn draw_bars<const N: usize>(fb: &mut Framebuffer, times: [u32; N], colors: [u16; N]) {
    assert!(N != 0 && N <= LCD_HEIGHT - 48);
    fb.clear();
    let fb = fb.pixels_mut();

    let longest = times.into_iter().max().unwrap_or(1).max(1);
    let slot = (LCD_HEIGHT - 48) / N;
    let height = slot.min(24);

    // All bars share one scale: longer means slower.
    for y in 24..LCD_HEIGHT - 24 {
        fb[y * LCD_WIDTH + 19] = WHITE;
        fb[y * LCD_WIDTH + LCD_WIDTH - 20] = GRAY;
    }

    for (index, time) in times.into_iter().enumerate() {
        let width = (u64::from(time) * (LCD_WIDTH - 40) as u64 / u64::from(longest)) as usize;
        let y = 24 + index * slot + (slot - height) / 2;
        for row in y..y + height {
            fb[row * LCD_WIDTH + 20..row * LCD_WIDTH + 20 + width].fill(colors[index]);
        }
    }
}
