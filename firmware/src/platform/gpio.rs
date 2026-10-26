use crate::platform::registers::Register32;

pub unsafe fn gpio_set(output_reg: &Register32, bits: u32) {
    let addr = output_reg.address() + 0x800;
    let val = (bits << 8) | bits;
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, val);
    }
}

pub unsafe fn gpio_clear(output_reg: &Register32, bits: u32) {
    let addr = output_reg.address() + 0x800;
    let val = bits << 8;
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, val);
    }
}
