use crate::platform::display::{LCD_HEIGHT, LCD_WIDTH};

const FONT_WIDTH: usize = 5;
const FONT_HEIGHT: usize = 7;
// each "font pixel" is a 4x4 square
const SCALE: usize = 4;
// always display 8 digits (complete u32)
const DIGIT_COUNT: usize = 8;
const DIGIT_ADVANCE: usize = (FONT_WIDTH + 1) * SCALE;
const ROW_HEIGHT: usize = (FONT_HEIGHT + 2) * SCALE;

// Each row has five bits. A 1 draws a pixel, a 0 leaves the background
const HEX_FONT: [[u8; FONT_HEIGHT]; 16] = [
    // 0
    [
        0b01110,
        0b10001,
        0b10001,
        0b10001,
        0b10001,
        0b10001,
        0b01110,
    ],
    // 1
    [
        0b00100,
        0b01100,
        0b00100,
        0b00100,
        0b00100,
        0b00100,
        0b01110,
    ],
    // 2
    [
        0b01110,
        0b10001,
        0b00001,
        0b00010,
        0b00100,
        0b01000,
        0b11111,
    ],
    // 3
    [
        0b11110,
        0b00001,
        0b00001,
        0b01110,
        0b00001,
        0b00001,
        0b11110,
    ],
    // 4
    [
        0b00010,
        0b00110,
        0b01010,
        0b10010,
        0b11111,
        0b00010,
        0b00010,
    ],
    // 5
    [
        0b11111,
        0b10000,
        0b10000,
        0b11110,
        0b00001,
        0b00001,
        0b11110,
    ],
    // 6
    [
        0b01110,
        0b10000,
        0b10000,
        0b11110,
        0b10001,
        0b10001,
        0b01110,
    ],
    // 7
    [
        0b11111,
        0b00001,
        0b00010,
        0b00100,
        0b01000,
        0b01000,
        0b01000,
    ],
    // 8
    [
        0b01110,
        0b10001,
        0b10001,
        0b01110,
        0b10001,
        0b10001,
        0b01110,
    ],
    // 9
    [
        0b01110,
        0b10001,
        0b10001,
        0b01111,
        0b00001,
        0b00001,
        0b01110,
    ],
    // A
    [
        0b01110,
        0b10001,
        0b10001,
        0b11111,
        0b10001,
        0b10001,
        0b10001,
    ],
    // B
    [
        0b11110,
        0b10001,
        0b10001,
        0b11110,
        0b10001,
        0b10001,
        0b11110,
    ],
    // C
    [
        0b01110,
        0b10001,
        0b10000,
        0b10000,
        0b10000,
        0b10001,
        0b01110,
    ],
    // D
    [
        0b11110,
        0b10001,
        0b10001,
        0b10001,
        0b10001,
        0b10001,
        0b11110,
    ],
    // E
    [
        0b11111,
        0b10000,
        0b10000,
        0b11110,
        0b10000,
        0b10000,
        0b11111,
    ],
    // F
    [
        0b11111,
        0b10000,
        0b10000,
        0b11110,
        0b10000,
        0b10000,
        0b10000,
    ],
];

pub fn draw_hex(fb: &mut [u16; LCD_WIDTH * LCD_HEIGHT], x: usize, y: usize, value: u32, color: u16) {
    assert!(x <= LCD_WIDTH - DIGIT_COUNT * DIGIT_ADVANCE);
    assert!(y <= LCD_HEIGHT - FONT_HEIGHT * SCALE);

    for digit_index in 0..DIGIT_COUNT {
        // 4 bits form one hex digit, so shift selected 4 bits to the right to process
        let shift = (DIGIT_COUNT - 1 - digit_index) * 4;
        let digit = ((value >> shift) & 0xF) as usize;
        let digit_x = x + digit_index * DIGIT_ADVANCE;
        draw_digit(fb, digit_x, y, digit, color);
    }
}

// one value per row: green if the check passed, red if not
pub fn draw_checks<const N: usize>(fb: &mut [u16; LCD_WIDTH * LCD_HEIGHT], checks: [(u32, bool); N]) {
    assert!(N != 0 && N * ROW_HEIGHT <= LCD_HEIGHT);
    fb.fill(0);

    let x = (LCD_WIDTH - DIGIT_COUNT * DIGIT_ADVANCE) / 2;
    let top = (LCD_HEIGHT - N * ROW_HEIGHT) / 2;
    for (index, (value, ok)) in checks.into_iter().enumerate() {
        let color = if ok { 0x07E0 } else { 0xF800 };
        draw_hex(fb, x, top + index * ROW_HEIGHT, value, color);
    }
}

fn draw_digit(fb: &mut [u16; LCD_WIDTH * LCD_HEIGHT], x: usize, y: usize, digit: usize, color: u16) {
    for (row, row_bits) in HEX_FONT[digit].iter().enumerate() {
        for column in 0..FONT_WIDTH {
            // bit mask
            let pixel_mask = 1 << (FONT_WIDTH - 1 - column);
            // skip if current "font pixel" should not be drawn
            if row_bits & pixel_mask == 0 {
                continue;
            }

            let pixel_x = x + column * SCALE;
            let pixel_y = y + row * SCALE;

            // Enlarge each "font pixel" into a SCALE-by-SCALE square
            for screen_y in pixel_y..pixel_y + SCALE {
                let start = screen_y * LCD_WIDTH + pixel_x;
                fb[start..start + SCALE].fill(color);
            }
        }
    }
}
