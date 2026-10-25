use core::ptr::read_volatile;

// start.s fills every stack with this before anything runs
const FILL: u32 = 0xDEADBEEF;

// stack tops from linker.ld: each stack starts where the one below it ends
unsafe extern "C" {
    static _stackbegin: u32;
    static _systemstack: u32;
    static _irqstack: u32;
    static _fiqstack: u32;
    static _exceptionstack: u32;
}

pub struct Usage {
    pub name: &'static str,
    pub used: usize,
    pub size: usize,
}

pub fn usage() -> [Usage; 4] {
    let bounds = [
        &raw const _stackbegin as usize,
        &raw const _systemstack as usize,
        &raw const _irqstack as usize,
        &raw const _fiqstack as usize,
        &raw const _exceptionstack as usize,
    ];
    let names = ["system", "irq", "fiq", "exception"];
    core::array::from_fn(|index| {
        let (begin, end) = (bounds[index], bounds[index + 1]);
        Usage { name: names[index], used: end - untouched_end(begin, end), size: end - begin }
    })
}

// stacks grow down, so the untouched fill is a run from the bottom up
fn untouched_end(begin: usize, end: usize) -> usize {
    let mut address = begin;
    while address < end && unsafe { read_volatile(address as *const u32) } == FILL {
        address += 4;
    }
    address
}
