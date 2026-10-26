use core::arch::asm;

use crate::platform::registers::{CLOCK_SOURCE, CPU_CTRL, DEV_INIT2, DEV_TIMING1, PLL_CONTROL, PLL_STATUS};

const INIT_PLL: u32 = 0x4000_0000;

#[derive(Clone, Copy)]
pub enum CpuFrequency {
    Mhz24,
    Mhz30,
    Mhz80,
}

// Startup only: CPU core, interrupts masked, COP asleep.
#[unsafe(link_section = ".icode.clock")]
#[inline(never)]
pub fn set_frequency(frequency: CpuFrequency) {
    unsafe {
        match frequency {
            CpuFrequency::Mhz24 => PLL_CONTROL.modify(|value| value | 0x0800_0000),
            CpuFrequency::Mhz30 | CpuFrequency::Mhz80 => {
                DEV_INIT2.modify(|value| value | INIT_PLL);
                PLL_CONTROL.modify(|value| value | 0x8800_0000);
            }
        }

        CPU_CTRL.write(0x4800_0003);
        asm!("nop", options(nomem, nostack, preserves_flags));
        CLOCK_SOURCE.write(0x2000_2222);
        DEV_TIMING1.write(0x0000_0303);
        CPU_CTRL.write(0x4800_001F);
        asm!("nop", options(nomem, nostack, preserves_flags));

        let (pll, timing) = match frequency {
            CpuFrequency::Mhz24 => {
                // The 24 MHz reference clock needs no PLL
                PLL_CONTROL.modify(|value| value & !0x8000_0000);
                DEV_INIT2.modify(|value| value & !INIT_PLL);
                return;
            }
            // PP5022: 24 MHz * 5 / 1 / 4 = 30 MHz
            CpuFrequency::Mhz30 => (0x8A22_0501, 0x0000_0303),
            // PP5022: 24 MHz * 20 / 3 / 2 = 80 MHz
            CpuFrequency::Mhz80 => (0x8A12_1403, 0x0000_0808),
        };

        PLL_CONTROL.write(pll);
        while PLL_STATUS.read() & 0x8000_0000 == 0 {
            core::hint::spin_loop();
        }

        CPU_CTRL.write(0x4800_0003);
        asm!("nop", options(nomem, nostack, preserves_flags));
        DEV_TIMING1.write(timing);
        CLOCK_SOURCE.write(0x2000_7777);
        CPU_CTRL.write(0x4800_001F);
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
