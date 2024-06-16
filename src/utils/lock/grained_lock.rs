use std::{ops::Deref, ptr::NonNull};

use dynstack::{dyn_push, DynStack};
use parking_lot::RwLock;

use super::{
    grained_cell::GrainedUnsafeCell,
    grained_ref::{Immutable, Mutable},
    Ref,
};

#[derive(Default)]
/// For internal use only.
/// 
/// Fine Grained Lock implementation.
/// Allow for fine grained locking mechanism with thread safety.
/// This is mainly used for nested locking data structure that allows
/// for locking without much hassle.
pub(crate) struct GrainedLock<T> {
    pub(crate) lock: RwLock<()>,
    pub(crate) data: GrainedUnsafeCell<T>,
}

impl<T> GrainedLock<T> {
    pub fn borrow<'a>(&'a self) -> Ref<'a, T, Immutable> {
        let mut vec: DynStack<dyn Deref<Target = ()>> = DynStack::new();
        dyn_push!(vec, self.lock.read());
        Ref::new(NonNull::new(self.data.0.get()).unwrap(), vec)
    }

    pub fn borrow_mut<'a>(&'a self) -> Ref<'a, T, Mutable> {
        let mut vec: DynStack<dyn Deref<Target = ()>> = DynStack::new();
        dyn_push!(vec, self.lock.write());
        Ref::new(NonNull::new(self.data.0.get()).unwrap(), vec)
    }

    pub fn take(self) -> T {
        // need to make sure there is no other borrow
        let _lock = self.lock.write();

        // simply return T
        self.data.0.into_inner()
    }

    pub(crate) fn new(data: T) -> Self {
        Self {
            lock: RwLock::new(()),
            data: GrainedUnsafeCell::new(data),
        }
    }
}

#[cfg(test)]
mod test_grained_lock {
    use crate::utils::lock::grained_lock::GrainedLock;

    #[test]
    fn test_send_and_sync() {
        let lock = GrainedLock::<i32>::default();
        let handle = std::thread::spawn(move || {
            assert_eq!(unsafe { *lock.data.0.get().as_ref().unwrap() }, 0);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_default() {
        let lock = GrainedLock::<i32>::default();
        assert_eq!(unsafe { *lock.data.0.get().as_ref().unwrap() }, 0);
    }

    #[test]
    fn test_new() {
        let lock = GrainedLock::<i32>::new(1);
        assert_eq!(unsafe { *lock.data.0.get().as_ref().unwrap() }, 1);
    }

    #[test]
    fn test_take() {
        let lock = GrainedLock::<i32>::new(1);
        assert_eq!(lock.take(), 1);
    }

    #[test]
    fn test_borrow() {
        let lock = GrainedLock::<i32>::default();
        let _ref = lock.borrow();
        assert_eq!(*_ref, 0);
    }

    #[test]
    fn test_borrow_mut() {
        let lock = GrainedLock::<i32>::default();
        let mut _ref = lock.borrow_mut();
        *_ref = 1;
        assert_eq!(*_ref, 1);
    }
}
