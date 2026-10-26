use core::arch::global_asm;
use core::ptr::{read_volatile, write_volatile};

use crate::platform::display::{LCD_HEIGHT, LCD_WIDTH};
use crate::platform::gpio::gpio_clear;
use crate::platform::registers::{
    BCM_CONTROL, BCM_DATA32, BCM_RD_ADDR32, BCM_WR_ADDR32, GPIOC_ENABLE, GPIOC_OUTPUT_EN,
    GPO32_ENABLE,
};

global_asm!(include_str!("transfer.s"), options(raw));

const BCMA_COMMAND: u32 = 0x1F8;
const BCMA_CMDPARAM: u32 = 0xE0000;
const BCMCMD_LCD_UPDATE: u32 = 0xFFFF0000;

#[derive(PartialEq, Eq)]
pub enum LcdState {
    Initial,
    UpdateRequested,
}

pub struct Lcd {
    state: LcdState,
}

pub fn init() -> Lcd {
    // the bootloader has already powered and initialized the BCM
    unsafe {
        GPO32_ENABLE.modify(|value| value | 0xC000);
        gpio_clear(&GPIOC_ENABLE, 0x80);
        GPIOC_ENABLE.modify(|value| value | 0x40);
        GPIOC_OUTPUT_EN.modify(|value| value & !0x40);
        GPO32_ENABLE.modify(|value| value & !1);
    }

    Lcd { state: LcdState::Initial }
}

impl Lcd {
    // Full-screen RGB565 little-endian bytes, with a four-byte-aligned source
    pub fn draw_direct(&mut self, image: &[u8]) {
        assert_eq!(image.len(), LCD_WIDTH * LCD_HEIGHT * 2);
        assert_eq!(image.as_ptr() as usize & 3, 0);

        bcm_write_addr(BCMA_CMDPARAM);
        unsafe { lcd_write_data_asm_raw(image.as_ptr().cast::<u16>(), image.len() / 2) };
        self.request_update();
    }
    pub fn update(&mut self, fb: &[u16; LCD_WIDTH * LCD_HEIGHT]) {
        self.update_rect(0, 0, LCD_WIDTH, LCD_HEIGHT, fb);
    }

    pub fn update_rect(&mut self, mut x: usize, y: usize, mut width: usize, mut height: usize, fb: &[u16; LCD_WIDTH * LCD_HEIGHT]) {
        if x >= LCD_WIDTH || y >= LCD_HEIGHT {
            return;
        }

        width = width.min(LCD_WIDTH - x);
        height = height.min(LCD_HEIGHT - y);
        if width == 0 || height == 0 {
            return;
        }

        // BCM requires an even starting column and an even width
        width = (width + (x & 1) + 1) & !1;
        x &= !1;

        let mut offset = y * LCD_WIDTH + x;
        let mut bcmaddr = BCMA_CMDPARAM + (offset * 2) as u32;

        if width == LCD_WIDTH {
            bcm_write_addr(bcmaddr);
            lcd_write_data(&fb[offset..offset + width * height]);
        }
        else {
            for _ in 0..height {
                bcm_write_addr(bcmaddr);
                lcd_write_data(&fb[offset..offset + width]);
                bcmaddr += (LCD_WIDTH * 2) as u32;
                offset += LCD_WIDTH;
            }
        }

        self.request_update();
    }

    fn request_update(&mut self) {
        if self.state == LcdState::UpdateRequested {
            loop {
                let command = bcm_read32(BCMA_COMMAND);
                if command != BCMCMD_LCD_UPDATE && command != 0xFFFF {
                    break;
                }
                core::hint::spin_loop();
            }
        }

        bcm_write32(BCMA_COMMAND, BCMCMD_LCD_UPDATE);
        unsafe { write_volatile(BCM_CONTROL, 0x31) };
        self.state = LcdState::UpdateRequested;
    }
}

fn bcm_write_addr(address: u32) {
    unsafe {
        BCM_WR_ADDR32.write(address);
        while read_volatile(BCM_CONTROL) & 0x2 == 0 {
            core::hint::spin_loop();
        }
    }
}

fn bcm_write32(address: u32, value: u32) {
    bcm_write_addr(address);
    unsafe { BCM_DATA32.write(value) };
}

fn bcm_read32(address: u32) -> u32 {
    unsafe {
        while read_volatile(BCM_RD_ADDR32.address() as *const u16) & 1 == 0 {
            core::hint::spin_loop();
        }

        BCM_RD_ADDR32.write(address);
        while read_volatile(BCM_CONTROL) & 0x10 == 0 {
            core::hint::spin_loop();
        }

        BCM_DATA32.read()
    }
}

unsafe extern "C" {
    fn lcd_write_data_asm_raw(pixels: *const u16, count: usize);
}

pub fn lcd_write_data(pixels: &[u16]) {
    assert_eq!(pixels.as_ptr() as usize & 3, 0);
    assert_eq!(pixels.len() & 1, 0);
    unsafe { lcd_write_data_asm_raw(pixels.as_ptr(), pixels.len()) };
}
