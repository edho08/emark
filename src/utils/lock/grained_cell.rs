use std::cell::UnsafeCell;

#[derive(Default, Debug)]
pub(crate) struct GrainedUnsafeCell<T>(pub(crate) UnsafeCell<T>);
unsafe impl<T> Sync for GrainedUnsafeCell<T> {}
unsafe impl<T> Send for GrainedUnsafeCell<T> {}

impl<T> GrainedUnsafeCell<T> {
    pub(crate) fn new(data: T) -> Self {
        Self(UnsafeCell::new(data))
    }
}

#[cfg(test)]
mod test_grained_unsafe_cell {
    use crate::utils::lock::grained_cell::GrainedUnsafeCell;

    #[test]
    fn test_send() {
        let cell = GrainedUnsafeCell::<i32>::default();
        let handle = std::thread::spawn(move || {
            assert_eq!(unsafe { *cell.0.get().as_ref().unwrap() }, 0);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_default() {
        let cell = GrainedUnsafeCell::<i32>::default();
        assert_eq!(unsafe { *cell.0.get().as_ref().unwrap() }, 0);
    }

    #[test]
    fn test_new() {
        let cell = GrainedUnsafeCell::<i32>::new(1);
        assert_eq!(unsafe { *cell.0.get().as_ref().unwrap() }, 1);
    }
}
