/// Event trait.
/// 
/// An empty trait that is used to define events.
/// Mainly used for type safety.
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
pub trait Event{}