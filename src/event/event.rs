/// Event trait.
///
/// An empty trait that is used to define events.
/// Mainly used for type safety.
///
/// To understand more about event see [event](crate::event)
///
/// # Examples
/// Implementing Event trait is trivial as shown below:
/// ```
/// use emark::prelude::Event;
///
/// struct SomeEvent;
///
/// impl Event for SomeEvent {}
/// ```
pub trait Event {}

pub(crate) struct GenericEvent;
impl Event for GenericEvent {}

#[cfg(test)]
mod test_event {
    use crate::event::event::Event;

    #[test]
    fn test_event() {
        #[allow(dead_code)]
        struct SomeEvent;
        impl Event for SomeEvent {}
    }
}
