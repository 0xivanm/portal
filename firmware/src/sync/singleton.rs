use core::cell::UnsafeCell;

pub struct Singleton<T> {
    value: UnsafeCell<T>,
    taken: UnsafeCell<bool>,
}

// only take() and steal() reach the value
unsafe impl<T: Send> Sync for Singleton<T> {}

impl<T> Singleton<T> {
    pub const fn new(value: T) -> Self {
        Self { value: UnsafeCell::new(value), taken: UnsafeCell::new(false) }
    }

    // not interrupt safe, so take it at startup
    pub fn take(&'static self) -> Option<&'static mut T> {
        let taken = unsafe { &mut *self.taken.get() };
        if *taken {
            return None;
        }
        *taken = true;
        Some(unsafe { self.steal() })
    }

    /// # Safety
    /// Aliases the owner's reference: only for code that never returns to it, like the crash screen.
    pub unsafe fn steal(&'static self) -> &'static mut T {
        unsafe { &mut *self.value.get() }
    }
}
