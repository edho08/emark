#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Ord)]
#[repr(u8)]
/// Event priority.
/// # Event Priority
///
/// The priority of an event determines the order in which events are
/// processed by the event loop. Events are processed in descending order
/// of priority. Events with higher priority are processed before events
/// with lower priority. The priority levels are as follows:
///
/// - `Interrupt`: The highest priority level. This is for
///   interrupts that require immediate attention. These events are
///   processed before any other events.
///
/// - `High`: High priority events are processed before
///   normal priority events. These events should be processed
///   promptly, but are not as time-critical as interrupt events.
///
/// - `Normal`: Normal priority events are the default
///   priority level
///
/// - `Routine`: Routine priority events are the lowest
///   priority level. These events are processed after all other
///   events. These events are used as self-refiring events where
///   an event has a handler that emit itself.
///
/// # Examples
pub enum Priority {
    Interrupt = 0,
    High = 1,
    #[default]
    Normal = 2,
    Routine = 3,
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other).reverse())
    }
}

impl From<u8> for Priority {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Interrupt,
            1 => Self::High,
            2 => Self::Normal,
            3 => Self::Routine,
            _ => unreachable!(),
        }
    }
}

impl From<Priority> for u8 {
    fn from(value: Priority) -> Self {
        value as u8
    }
}

impl From<Priority> for usize {
    fn from(value: Priority) -> Self {
        value as usize
    }
}

impl Iterator for Priority {
    type Item = Priority;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Priority::Interrupt => Some(Priority::High),
            Priority::High => Some(Priority::Normal),
            Priority::Normal => Some(Priority::Routine),
            Priority::Routine => Some(Priority::Interrupt),
        }
    }
}

impl From<Interrupt> for Priority {
    fn from(_: Interrupt) -> Self {
        Self::Interrupt
    }
}

impl From<High> for Priority {
    fn from(_: High) -> Self {
        Self::High
    }
}

impl From<Normal> for Priority {
    fn from(_: Normal) -> Self {
        Self::Normal
    }
}

impl From<Routine> for Priority {
    fn from(_: Routine) -> Self {
        Self::Routine
    }
}

/// Priority type state trait.
/// This is the same as `Priority` but defined using type state method.
///
/// # Examples
pub trait PriorityState: Copy + Clone {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Interrupt type state priority.
pub struct Interrupt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// High type state priority.
pub struct High;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Normal type state priority.
pub struct Normal;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Routine type state priority.
pub struct Routine;

impl PriorityState for Interrupt {}
impl PriorityState for High {}
impl PriorityState for Normal {}
impl PriorityState for Routine {}

#[cfg(test)]
mod test_priority {
    use crate::event::priority::{High, Interrupt, Normal, Priority, Routine};

    #[test]
    fn test_from_u8() {
        assert_eq!(Priority::from(0), Priority::Interrupt);
        assert_eq!(Priority::from(1), Priority::High);
        assert_eq!(Priority::from(2), Priority::Normal);
        assert_eq!(Priority::from(3), Priority::Routine);
    }

    #[test]
    fn test_from_priority() {
        assert_eq!(u8::from(Priority::Interrupt), 0u8);
        assert_eq!(u8::from(Priority::High), 1u8);
        assert_eq!(u8::from(Priority::Normal), 2u8);
        assert_eq!(u8::from(Priority::Routine), 3u8);
    }

    #[test]
    fn test_priority_order() {
        assert!(Priority::Interrupt > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Routine);
        assert!(Priority::Routine < Priority::Interrupt);
    }

    #[test]
    fn test_priority_iterator() {
        let mut priority = Priority::Interrupt;
        assert_eq!(priority.next(), Some(Priority::High));
        let mut priority = priority.next().unwrap();
        assert_eq!(priority.next(), Some(Priority::Normal));
        let mut priority = priority.next().unwrap();
        assert_eq!(priority.next(), Some(Priority::Routine));
        let mut priority = priority.next().unwrap();
        assert_eq!(priority.next(), Some(Priority::Interrupt));
    }

    #[test]
    fn test_priority_from_type_state() {
        assert_eq!(Priority::from(Interrupt), Priority::Interrupt);
        assert_eq!(Priority::from(High), Priority::High);
        assert_eq!(Priority::from(Normal), Priority::Normal);
        assert_eq!(Priority::from(Routine), Priority::Routine);
    }
}