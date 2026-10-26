use crate::platform::gpio::{gpio_clear, gpio_set};
use crate::platform::registers::GPIOL_OUTPUT_VAL;

const BACKLIGHT_BIT: u32 = 0x80;

pub fn on() {
    unsafe { gpio_set(&GPIOL_OUTPUT_VAL, BACKLIGHT_BIT) };
}

pub fn off() {
    unsafe { gpio_clear(&GPIOL_OUTPUT_VAL, BACKLIGHT_BIT) };
}
