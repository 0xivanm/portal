use crate::platform::registers::USEC_TIMER;

#[inline(always)]
pub fn micros() -> u32 {
    unsafe { USEC_TIMER.read() }
}

pub fn wait_until(deadline: u32) {
    while (micros().wrapping_sub(deadline) as i32) < 0 {
        core::hint::spin_loop();
    }
}

pub fn delay_us(us: u32) {
    wait_until(micros().wrapping_add(us));
}

pub fn delay_ms(ms: u32) {
    // chuncked waits to avoid overflowing conversion to ms
    let mut remaining = ms;
    while remaining != 0 {
        let chunk = remaining.min(1_000);
        wait_until(micros().wrapping_add(chunk * 1_000));
        remaining -= chunk;
    }
}
