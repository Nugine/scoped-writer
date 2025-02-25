#![allow(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

mod mut_cell;
use self::mut_cell::MutCell;

use std::io::Write;
use std::mem;
use std::ptr::NonNull;

type Trait<'a> = dyn Write + 'a;

thread_local! {
    static SLOT: MutCell<Option<NonNull<Trait<'static>>>> = const { MutCell::new(None) };
}

/// Sets the global writer for the duration of the closure in current thread.
pub fn scoped<R>(w: &mut dyn Write, f: impl FnOnce() -> R) -> R {
    struct Guard(Option<NonNull<Trait<'static>>>);

    impl Drop for Guard {
        fn drop(&mut self) {
            SLOT.with(|slot| slot.with(|ptr| *ptr = self.0));
        }
    }

    let cur: NonNull<Trait<'static>> = unsafe { mem::transmute(NonNull::from(w)) };
    let prev = SLOT.with(|slot| slot.with(|ptr| ptr.replace(cur)));
    let _guard = Guard(prev);

    f()
}

/// Executes a closure with the global writer, skips if the writer is not set.
///
/// Reentrancy is not allowed.
///
/// # Panics
/// Panics if this function is called recursively.
pub fn with<R>(f: impl FnOnce(&mut dyn Write) -> R) -> Option<R> {
    SLOT.with(|slot| slot.with(|ptr| (*ptr).map(|mut p| f(unsafe { p.as_mut() }))))
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
