use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};


/// Mutex-like wrapper, but it actually does not perform any locking.
///
/// Use this wrapper when:
///   1. [`Send`], [`Sync`] and the interior mutability is needed,
///   2. it is (manually) guaranteed that data races will not occur, and
///   3. the performance is critical.
///
/// **Note**: This wrapper completely bypasses the "shared XOR mutable" rule of Rust.
/// Therefore, using this wrapper is **extremely unsafe** and should be avoided whenever possible.
#[derive(Debug)]
#[repr(transparent)]
pub struct MutexLike<T: ?Sized> {
    data: UnsafeCell<T>,
}

/// Smart pointer like wrapper that is returned when [`MutexLike`] is "locked".
#[derive(Debug)]
pub struct MutexGuardLike<'a, T: ?Sized + 'a> {
    mutex: &'a MutexLike<T>,
}

unsafe impl<T: ?Sized + Send> Send for MutexLike<T> {}
unsafe impl<T: ?Sized + Send> Sync for MutexLike<T> {}
unsafe impl<'a, T: ?Sized + Sync + 'a> Sync for MutexGuardLike<'a, T> {}

impl<T> MutexLike<T> {
    /// Creates a new [`MutexLike`] with the given value.
    ///
    /// # Examples
    /// ```
    /// use postflop_solver::MutexLike;
    ///
    /// let mutex_like = MutexLike::new(0);
    /// ```
    #[inline]
    pub fn new(val: T) -> Self {
        Self {
            data: UnsafeCell::new(val),
        }
    }
}

impl<T: ?Sized> MutexLike<T> {
    /// Acquires a mutex-like object **without** performing any locking.
    ///
    /// # Examples
    /// ```
    /// use postflop_solver::MutexLike;
    ///
    /// let mutex_like = MutexLike::new(0);
    /// *mutex_like.lock() = 10;
    /// assert_eq!(*mutex_like.lock(), 10);
    /// ```
    #[inline]
    pub fn lock(&self) -> MutexGuardLike<T> {
        MutexGuardLike { mutex: self }
    }
}

impl<T: ?Sized + Default> Default for MutexLike<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<'a, T: ?Sized + 'a> Deref for MutexGuardLike<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized + 'a> DerefMut for MutexGuardLike<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}
