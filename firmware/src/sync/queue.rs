use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::{read_volatile, write_volatile};

// Lock-free queue for one producer and one consumer on the same core, like an
// interrupt and main. Each side only writes its own index, after the slot it
// guards, so neither ever sees a half-written value and nothing has to mask
// interrupts. N must be a power of two so that wrapping indices keeps the slot order.
pub struct Queue<T, const N: usize> {
    slots: UnsafeCell<[MaybeUninit<T>; N]>,
    // next slot to write, only changed by the producer
    head: UnsafeCell<usize>,
    // next slot to read, only changed by the consumer
    tail: UnsafeCell<usize>,
    // values pushed while full, only changed by the producer
    dropped: UnsafeCell<u32>,
}

unsafe impl<T: Send, const N: usize> Sync for Queue<T, N> {}

impl<T: Copy, const N: usize> Queue<T, N> {
    const MASK: usize = {
        assert!(N.is_power_of_two(), "queue length must be a power of two");
        N - 1
    };

    pub const fn new() -> Self {
        Self {
            slots: UnsafeCell::new([const { MaybeUninit::uninit() }; N]),
            head: UnsafeCell::new(0),
            tail: UnsafeCell::new(0),
            dropped: UnsafeCell::new(0),
        }
    }

    /// Drops the value and counts it when the queue is full.
    ///
    /// # Safety
    /// Only one context may ever push.
    pub unsafe fn push(&self, value: T) {
        unsafe {
            let head = read_volatile(self.head.get());
            if head.wrapping_sub(read_volatile(self.tail.get())) == N {
                write_volatile(self.dropped.get(), read_volatile(self.dropped.get()).wrapping_add(1));
                return;
            }
            write_volatile(self.slot(head), MaybeUninit::new(value));
            write_volatile(self.head.get(), head.wrapping_add(1));
        }
    }

    /// # Safety
    /// Only one context may ever pop.
    pub unsafe fn pop(&self) -> Option<T> {
        unsafe {
            let tail = read_volatile(self.tail.get());
            if tail == read_volatile(self.head.get()) {
                return None;
            }
            // every slot between tail and head was written by push
            let value = read_volatile(self.slot(tail)).assume_init();
            write_volatile(self.tail.get(), tail.wrapping_add(1));
            Some(value)
        }
    }

    pub fn dropped(&self) -> u32 {
        unsafe { read_volatile(self.dropped.get()) }
    }

    fn slot(&self, index: usize) -> *mut MaybeUninit<T> {
        unsafe { (self.slots.get() as *mut MaybeUninit<T>).add(index & Self::MASK) }
    }
}
