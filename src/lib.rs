#![allow(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

use std::cell::RefCell;
use std::io;
use std::ptr;

type SlotType = *mut dyn io::Write;

thread_local! {
    static SLOT: RefCell<*mut SlotType> = const { RefCell::new(ptr::null_mut()) };
}

struct SlotGuard(*mut SlotType);

impl SlotGuard {
    #[must_use]
    fn new(cur: *mut SlotType) -> Self {
        let prev = SLOT.with(|slot| slot.replace(cur));
        SlotGuard(prev)
    }
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        SLOT.with(|slot| *slot.borrow_mut() = self.0);
    }
}

/// Sets the global writer for the duration of the closure in current thread.
pub fn scoped<R>(mut w: &mut dyn io::Write, f: impl FnOnce() -> R) -> R {
    let _guard = SlotGuard::new(ptr::addr_of_mut!(w).cast());
    f()
}

/// Executes a closure with the global writer, skips if the writer is not set.
///
/// Reentrancy is not allowed.
///
/// # Panics
/// Panics if this function is called recursively
pub fn with<R>(f: impl FnOnce(&mut dyn io::Write) -> R) -> Option<R> {
    SLOT.with(|slot| {
        let Ok(cur) = slot.try_borrow_mut() else {
            panic!("Reentrancy is not allowed")
        };
        let p = cur.cast::<&mut dyn io::Write>();
        if p.is_null() {
            None
        } else {
            Some(f(unsafe { &mut **p }))
        }
    })
}

/// [`writeln!`] to the global writer.
#[macro_export]
macro_rules! g {
    () => {{
        $crate::with(|w|writeln!(w).unwrap());
    }};
    ($fmt:literal $($arg:tt)*) => {{
        $crate::with(|w|writeln!(w, $fmt $($arg)*).unwrap());
    }};
}

/// Writes lines to the global writer.
pub fn g<L>(lines: impl AsRef<[L]>)
where
    L: AsRef<str>,
{
    with(|w| {
        for line in lines.as_ref() {
            writeln!(w, "{}", line.as_ref()).unwrap();
        }
    });
}
