use super::GrainedLock;
use dynstack::{dyn_push, DynStack};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[allow(dead_code)]
pub trait LockState {}
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Immutable;
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Mutable;

impl LockState for Immutable {}
impl LockState for Mutable {}

pub struct Ref<'a, T, S>
where
    S: LockState,
{
    locks: DynStack<dyn Deref<Target = ()> + 'a>,
    data: NonNull<T>,
    _marker: PhantomData<S>,
}

impl<'a, T, S> Ref<'a, GrainedLock<T>, S>
where
    S: LockState,
    T: 'static,
{
    pub fn borrow(self) -> Ref<'a, T, Immutable> {
        // destructure ref
        let Ref {
            mut locks,
            data,
            _marker,
        } = self;

        // get immutable reference to the grained lock
        // here we are turning a non-null pointer into a reference
        // since we know it is non-null, we can safely dereference it
        let grained = unsafe { data.as_ref() };

        // push the grained lock into the stack
        // this operation allows for the grained lock
        dyn_push!(locks, grained.lock.read());

        // get the new data from the grained lock
        let data = NonNull::new(grained.data.0.get()).unwrap();

        // reconstruct ref
        Ref {
            locks,
            data,
            _marker: PhantomData::<Immutable>,
        }
    }

    pub fn borrow_mut(self) -> Ref<'a, T, Mutable> {
        // destructure ref
        let Ref {
            mut locks,
            mut data,
            _marker,
        } = self;

        // get mutable reference to the grained lock
        // here we are turning a non-null pointer into a reference
        // since we know it is non-null, we can safely dereference it
        let grained = unsafe { data.as_mut() };

        // push the grained lock into the stack
        // this operation allows for the grained lock
        dyn_push!(locks, grained.lock.write());

        // get the new data from the grained lock
        let data = NonNull::new(grained.data.0.get()).unwrap();

        // reconstruct ref
        Ref {
            locks,
            data,
            _marker: PhantomData::<Mutable>,
        }
    }
}

impl<'a, T, S> Deref for Ref<'a, T, S>
where
    S: LockState,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}

impl<'a, T> DerefMut for Ref<'a, T, Mutable> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.as_mut() }
    }
}

impl<'a, T, S> AsRef<T> for Ref<'a, T, S>
where
    S: LockState,
{
    fn as_ref(&self) -> &T {
        self
    }
}

impl<'a, T> AsMut<T> for Ref<'a, T, Mutable> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T, S> Ref<'a, T, S>
where
    S: LockState,
{
    pub fn map_cell<K, F: FnOnce(&T) -> &GrainedLock<K>>(self, f: F) -> Ref<'a, GrainedLock<K>, S> {
        let Ref {
            locks,
            data,
            _marker,
        } = self;
        let data = unsafe { f(data.as_ref()) } as *const GrainedLock<K> as *mut GrainedLock<K>;

        Ref {
            locks,
            data: NonNull::new(data).unwrap(),
            _marker,
        }
    }

    pub fn map<
        K,
        F: FnMut(&T) -> (&K, Option<NonNull<dyn Deref<Target = ()> + 'a>>),
        NS: LockState,
    >(
        self,
        mut f: F,
    ) -> Ref<'a, K, NS> {
        let Ref {
            mut locks,
            data,
            _marker,
        } = self;
        let (data, lock) = unsafe { f(data.as_ref()) };
        if let Some(lock) = lock {
            unsafe {
                locks.push(lock.as_ptr());
            }
            // core::mem::forget(lock)
        }
        Ref {
            locks,
            data: NonNull::from(data),
            _marker: PhantomData::<NS>,
        }
    }

    pub unsafe fn leak(self) -> *mut T {
        self.data.as_ptr()
    }

    pub fn new(data: NonNull<T>, locks: DynStack<dyn Deref<Target = ()> + 'a>) -> Self {
        Self {
            locks: locks,
            data: data,
            _marker: PhantomData::<S>,
        }
    }
}

#[cfg(test)]
mod test_grained_ref {
    use crate::utils::lock::grained_lock::GrainedLock;

    use super::Immutable;

    #[test]
    fn test_map_cell() {
        let resource = GrainedLock::new(Vec::<GrainedLock<i32>>::new());
        resource.borrow_mut().push(GrainedLock::default());

        let inner = resource
            .borrow()
            .map_cell(|vec| vec.get(0).unwrap())
            .borrow();

        assert_eq!(*inner, i32::default());
    }

    #[test]
    fn test_map() {
        let resource = GrainedLock::new(Vec::<i32>::new());
        resource.borrow_mut().push(i32::default());

        let inner = resource
            .borrow()
            .map::<_, _, Immutable>(|vec| (vec.get(0).unwrap(), None));

        assert_eq!(*inner, i32::default());
    }

    #[test]
    fn test_leak() {
        let resource = GrainedLock::new(Vec::<i32>::new());
        let _inner = unsafe { resource.borrow().leak() };

        assert_eq!(*resource.borrow(), Vec::<i32>::new());
    }

    #[test]
    fn test_as_ref() {
        let resource = GrainedLock::new(Vec::<i32>::new());
        let inner = resource.borrow();
        let inner = inner.as_ref();
        assert_eq!(*inner, Vec::<i32>::new());
    }

    #[test]
    fn test_as_mut() {
        let resource = GrainedLock::new(Vec::<i32>::new());
        let mut inner = resource.borrow_mut();
        let inner = inner.as_mut();
        assert_eq!(*inner, Vec::<i32>::new());
    }
}
