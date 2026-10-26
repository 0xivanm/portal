use crate::platform::registers::{DEV_EN, DEV_INIT1, PWM0_CTRL};
use crate::platform::timer;

const DEV_PWM: u32 = 0x0002_0000;
const PWM_ENABLE: u32 = 0x8000_0000;
// inferred carrier: 93,750 Hz / (divider + 1).
const PCM_DIVIDER: u32 = 0;

pub fn init() {
    unsafe {
        DEV_INIT1.modify(|value| value & !0xC as u32);
        DEV_EN.modify(|value| value | DEV_PWM);
    }
}

/// unsigned PCM sample, 128 is the waveform midpoint
#[inline(always)]
pub fn write_sample(sample: u8) {
    // rockbox form: bits 23:16, duty = sample / 256.
    let control = PWM_ENABLE | (u32::from(sample) << 16) | PCM_DIVIDER;
    unsafe { PWM0_CTRL.write(control) }
}

pub fn stop() {
    unsafe { PWM0_CTRL.write(0) }
}


const SAMPLE_INTERVAL_US: u32 = 125;

pub fn play_pcm_u8(samples: &[u8]) {
    let mut deadline = timer::micros();

    for &sample in samples {
        write_sample(sample);
        deadline = deadline.wrapping_add(SAMPLE_INTERVAL_US);
        timer::wait_until(deadline);
    }
    stop();
}