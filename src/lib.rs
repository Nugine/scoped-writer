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
        let prev = SLOT.replace(cur);
        SlotGuard(prev)
    }
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        SLOT.set(self.0);
    }
}

pub fn scoped<R>(mut w: &mut dyn io::Write, f: impl FnOnce() -> R) -> R {
    let _guard = SlotGuard::new((&raw mut w).cast());
    f()
}

#[allow(clippy::missing_panics_doc)]
pub fn with<R>(f: impl FnOnce(&mut dyn io::Write) -> R) -> Option<R> {
    SLOT.with(|slot| {
        let Ok(cur) = slot.try_borrow_mut() else {
            panic!("Reentrancy detected")
        };
        let p = cur.cast::<&mut dyn io::Write>();
        if p.is_null() {
            None
        } else {
            Some(f(unsafe { &mut **p }))
        }
    })
}

#[macro_export]
macro_rules! g {
    () => {
        use ::std::io::Write;
        $crate::with(|w|writeln!(w).unwrap())
    };
    ($fmt:literal $($arg:tt)*) => {{
        use ::std::io::Write;
        $crate::with(|w|writeln!(w, $fmt $($arg)*).unwrap())
    }};
}
