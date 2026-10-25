// RGB565: 5 bits red, 6 bits green, 5 bits blue
pub const fn rgb(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

pub const BLACK: u16 = rgb(0, 0, 0);
pub const WHITE: u16 = rgb(255, 255, 255);
pub const GRAY: u16 = rgb(64, 64, 64);
pub const RED: u16 = rgb(255, 0, 0);
pub const GREEN: u16 = rgb(0, 255, 0);
pub const BLUE: u16 = rgb(0, 0, 255);
