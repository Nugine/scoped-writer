use std::cell::Cell;
use std::cell::UnsafeCell;
use std::ops::Not;

pub struct MutCell<T> {
    value: UnsafeCell<T>,
    borrowing: Cell<bool>,
}

impl<T> MutCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            borrowing: Cell::new(false),
        }
    }

    pub fn with<R>(&self, f: impl for<'a> FnOnce(&'a mut T) -> R) -> R {
        struct Guard<'a>(&'a Cell<bool>);

        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }

        assert!(self.borrowing.replace(true).not(), "reentrancy detected");

        let _guard = Guard(&self.borrowing);

        f(unsafe { &mut *self.value.get() })
    }
}
