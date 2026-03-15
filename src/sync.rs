use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// A simple mutex implementation using a spin lock.
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    lock: AtomicBool,
}

impl<T> Mutex<T> {
    /// Creates a new `Mutex` wrapping an instance of `T`.
    pub const fn new(t: T) -> Self {
        Self {
            inner: UnsafeCell::new(t),
            lock: AtomicBool::new(false),
        }
    }

    /// Obtains an exclusive lock and then executes the given function, which is passed a mutable
    /// reference to the contained object, and returns its result. This function blocks execution
    /// until the lock is obtained.
    pub fn with_locked<F, U>(&self, func: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        // TODO: eventually we can invoke the scheduler on each acquire loop
        while self.lock.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }

        // SAFETY: We now hold the lock, so we have exclusive access to inner.
        let ret = func(unsafe { &mut *self.inner.get() });

        self.lock.store(false, Ordering::Release);
        ret
    }
}

unsafe impl<T> Sync for Mutex<T> {}

/// A wrapper for anything that must be initialized once at runtime. `Initializer` is intended to be
/// used for defining static objects that cannot be fully initialized at compile time.
pub struct Initializer<T> {
    inner: Mutex<Option<UnsafeCell<T>>>,
}

impl<T> Initializer<T> {
    /// Creates a new `Initializer` wrapping an instance of `T`.
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Initializes this instance by executing a function that returns the object to be stored
    /// inside. This function may only be called once on any instance, and panics if called again on
    /// an already initialized instance.
    pub fn initialize<F>(&self, func: F)
    where
        F: FnOnce() -> T,
    {
        self.inner.with_locked(|inner| {
            match inner {
                Some(_) => panic!("tried to call initialize more than once"),
                None => inner.insert(UnsafeCell::new(func())),
            };
        });
    }

    /// Returns an immutable reference to the object contained within this `Initializer` instance.
    /// Panics if this instance has not yet been initialized.
    pub fn get_ref(&self) -> &T {
        self.inner.with_locked(|inner| {
            match inner {
                // SAFETY: The mutex ensures no other thread currently has a mutable reference, and
                // if inner is Some then the contained value will never be mutated again.
                Some(cell) => unsafe { &*cell.get() },
                None => panic!("tried to call get_ref before initialization"),
            }
        })
    }
}

unsafe impl<T> Sync for Initializer<T> {}
