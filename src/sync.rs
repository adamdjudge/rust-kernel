use core::cell::UnsafeCell;
use core::ops::Deref;
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

/// A wrapper for anything that must be initialized once at runtime, such as global statics that
/// cannot be const initialized. It implements `Deref` to provide access to the inner `T` value.
///
/// `LazyInit<T>` is instantiated with a function that returns an initialized `T` value. This
/// function is transparently called the first time any thread dereferences the instance at runtime,
/// with an internal mutex ensuring that initialization occurs exactly once and that no thread
/// obtains a `T` reference before initialization is completed.
pub struct LazyInit<T> {
    inner: Mutex<Option<UnsafeCell<T>>>,
    func: fn() -> T,
}

impl<T> LazyInit<T> {
    /// Creates a new `LazyInit` instance, given a function that returns `T`. The function will be
    /// called once to initialize the contained value the first time this instance is dereferenced.
    pub const fn new(f: fn() -> T) -> Self {
        Self {
            inner: Mutex::new(None),
            func: f,
        }
    }
}

impl<T> Deref for LazyInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.with_locked(|inner| {
            let cell = inner.get_or_insert(UnsafeCell::new((self.func)()));
            // SAFETY: Inner is guaranteed to be initialized here
            unsafe { &*cell.get() }
        })
    }
}

unsafe impl<T> Sync for LazyInit<T> {}
