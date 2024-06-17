use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::utils::lock::GrainedLock;

use super::{
    priority::{Priority, PriorityState},
    Event,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy, Ord)]
pub(crate) struct EmittedEventInfo {
    priority: Priority,
    event_type_id: TypeId,
    vec_type_id: TypeId,
}

impl PartialOrd for EmittedEventInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority.cmp(other.priority))
    }
}

#[derive(Default, Debug)]
/// # EventManager
///
/// The `EventManager` is responsible for managing and deferring events until their
/// corresponding event handlers are ready to execute them. This mechanism ensures
/// that events are processed in an organized and efficient manner, enhancing the
/// performance of the event system.
///
/// When an event is fired, the `EventManager` does not execute it immediately.
/// Instead, it queues the event and defers its execution until the system signals
/// readiness. Events are processed in batches, where handlers receive a collection
/// of events of the same type, allowing for efficient batch processing.
///
/// The `EventManager` supports priority-based event handling. Events can be assigned
/// priorities, and the `EventManager` ensures that higher-priority events are processed
/// before lower-priority ones.
///
/// # Examples
///
pub struct EventManager {
    events: GrainedLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    events_set: GrainedLock<HashMap<TypeId, Priority>>,
    events_bus: GrainedLock<[Vec<EmittedEventInfo>; 4]>,
}

impl EventManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emits an event with the specified priority.
    ///
    /// This function takes an event of type `T` and a priority as input. It adds the event
    /// to the event manager's queue and sets its priority. The event will be processed
    /// when `System` is ready to execute and the event is at the top of the queue. 
    ///
    /// Always returns `Some(TypeId)` of the event that was emitted.
    pub fn emit_priority<T: Event + Send + Sync + 'static>(
        &self,
        event: T,
        priority: Priority,
    ) -> Option<TypeId> {
        // get type id of event
        let event_type_id = TypeId::of::<T>();
        // get vec id of event
        let vec_type_id = TypeId::of::<Vec<T>>();
        // get live events
        let mut events = self.events.borrow_mut();
        let events = events
            .entry(event_type_id)
            .or_insert(Box::new(Vec::<T>::new()) as Box<dyn Any + Send + Sync>)
            .downcast_mut::<Vec<T>>()
            .unwrap();

        // insert event
        events.push(event);

        // check if event_set already contains event.
        let mut event_set = self.events_set.borrow_mut();
        if let Some(old_priority) = event_set.get_mut(&event_type_id) {
            // event has already been fired beforehand
            // check if priority needs an upgrade
            if priority > *old_priority {
                // update priority from events_bus
                // get old index
                let index = usize::from(*old_priority);
                // get olf info
                let mut info = {
                    // get event bus
                    let mut events_bus = self.events_bus.borrow_mut();
                    let events = events_bus.get_mut(index).unwrap();
                    // get info index
                    let info_index = events
                        .iter()
                        .enumerate()
                        .find(|(_, info)| info.event_type_id == event_type_id)
                        .map(|(index, _)| index)
                        .unwrap();

                    // remove old info
                    events.remove(info_index)
                };

                // set new priority
                *old_priority = priority;
                info.priority = priority;

                // insert new info
                self.events_bus.borrow_mut()[index].push(info);
            }
        } else {
            // event has not been fired before
            // insert new priority
            event_set.insert(event_type_id, priority);
            // insert new info
            self.events_bus
                .borrow_mut()
                .get_mut(usize::from(priority))
                .unwrap()
                .push(EmittedEventInfo {
                    priority,
                    event_type_id,
                    vec_type_id,
                });
        }

        // return event type id
        Some(event_type_id)
    }

    /// Emits an event with normal priority.
    ///
    /// Always returns `Some(TypeId)` of the event that was emitted.
    pub fn emit<T: Event + Send + Sync + 'static>(&self, event: T) -> Option<TypeId> {
        self.emit_priority(event, Priority::Normal)
    }

    /// Emits and event like `emit_priority` but with a priority type state.
    /// for the supported type state see [PriorityState](crate::event::priority::PriorityState)
    pub fn emit_type_state<T: Event + Send + Sync + 'static, P: PriorityState>(
        &self,
        event: T,
    ) -> Option<TypeId> {
        self.emit_priority(event, P::priority())
    }

    // get next events to be executed.
    // returns None if no events are available.
    pub(crate) fn next_execution(
        &self,
    ) -> Option<Vec<(EmittedEventInfo, Box<dyn Any + Send + Sync>)>> {
        // get first available priority
        let mut priority = None;
        for (index, infos) in self.events_bus.borrow_mut().iter_mut().enumerate() {
            if !infos.is_empty() {
                priority = Some(Priority::from(index as u8));
                break;
            }
        }

        if let Some(priority) = priority {
            // get infos
            let infos = std::mem::take(&mut self.events_bus.borrow_mut()[usize::from(priority)])
                .into_iter()
                .map(|info| {
                    // get events
                    let event = self
                        .events
                        .borrow_mut()
                        .remove(&info.event_type_id)
                        .unwrap();

                    // remove event_set
                    self.events_set
                        .borrow_mut()
                        .remove(&info.event_type_id)
                        .unwrap();

                    // return info
                    (info, event)
                });

            // return infos
            return Some(infos.collect());
        }
        None
    }
}

#[cfg(test)]
mod test_event_manager {
    use super::*;
    use crate::event::{event::GenericEvent, event_manager::EventManager, priority::Interrupt};

    #[test]
    fn test_event_manager_new() {
        let event_manager = EventManager::new();
        assert_eq!(event_manager.events_bus.borrow().len(), 4);
    }

    #[test]
    fn test_event_manager_emit() {
        let event_manager = EventManager::new();
        assert_eq!(
            event_manager.emit(GenericEvent),
            Some(TypeId::of::<GenericEvent>())
        );
    }

    #[test]
    fn test_event_manager_emit_priority() {
        let event_manager = EventManager::new();
        assert_eq!(
            event_manager.emit_priority(GenericEvent, Priority::Interrupt),
            Some(TypeId::of::<GenericEvent>())
        );

        assert!(event_manager
            .events_set
            .borrow()
            .contains_key(&TypeId::of::<GenericEvent>()));

        assert_eq!(
            event_manager
                .events_set
                .borrow()
                .get(&TypeId::of::<GenericEvent>()),
            Some(&Priority::Interrupt)
        );

        assert_eq!(
            event_manager
                .events_bus
                .borrow()
                .get(usize::from(Priority::Interrupt))
                .unwrap()
                .len(),
            1
        );

        assert!(event_manager
            .events
            .borrow()
            .contains_key(&TypeId::of::<GenericEvent>()),);
    }

    #[test]
    fn test_event_manager_emit_type_state() {
        let event_manager = EventManager::new();
        assert_eq!(
            event_manager.emit_type_state::<GenericEvent, Interrupt>(GenericEvent),
            Some(TypeId::of::<GenericEvent>())
        );
    }

    #[test]
    fn test_event_manager_upgrade_event() {
        let event_manager = EventManager::new();
        assert!(event_manager.next_execution().is_none());

        // insert events
        event_manager
            .emit_priority(GenericEvent, Priority::Interrupt)
            .unwrap();
        event_manager
            .emit_priority(GenericEvent, Priority::Normal)
            .unwrap();
        event_manager
            .emit_priority(GenericEvent, Priority::Routine)
            .unwrap();
        event_manager
            .emit_priority(GenericEvent, Priority::High)
            .unwrap();

        // assert next execution is interrupt
        assert_eq!(
            event_manager
                .next_execution()
                .unwrap()
                .first()
                .unwrap()
                .0
                .priority,
            Priority::Interrupt
        );

        // assert next execution is None
        // note that this is due to the same event type fired multiple times, but with different priorities
        // which means it will only be executed once with highest priority
        assert!(event_manager.next_execution().is_none());
    }

    #[test]
    fn test_event_manager_next_execution() {
        struct TestEventInterrupt;
        struct TestEventHigh;
        struct TestEventNormal;
        struct TestEventRoutine;

        impl Event for TestEventInterrupt {}
        impl Event for TestEventHigh {}
        impl Event for TestEventNormal {}
        impl Event for TestEventRoutine {}

        let event_manager = EventManager::new();
        assert!(event_manager.next_execution().is_none());

        // insert events
        event_manager
            .emit_priority(TestEventInterrupt, Priority::Interrupt)
            .unwrap();
        event_manager
            .emit_priority(TestEventHigh, Priority::High)
            .unwrap();
        event_manager
            .emit_priority(TestEventNormal, Priority::Normal)
            .unwrap();
        event_manager
            .emit_priority(TestEventRoutine, Priority::Routine)
            .unwrap();

        // assert next execution is interrupt
        assert_eq!(
            event_manager
                .next_execution()
                .unwrap()
                .first()
                .unwrap()
                .0
                .priority,
            Priority::Interrupt
        );

        // assert next execution is high
        assert_eq!(
            event_manager
                .next_execution()
                .unwrap()
                .first()
                .unwrap()
                .0
                .priority,
            Priority::High
        );

        // assert next execution is normal
        assert_eq!(
            event_manager
                .next_execution()
                .unwrap()
                .first()
                .unwrap()
                .0
                .priority,
            Priority::Normal
        );

        // assert next execution is routine
        assert_eq!(
            event_manager
                .next_execution()
                .unwrap()
                .first()
                .unwrap()
                .0
                .priority,
            Priority::Routine
        );

        // assert next execution is None
        assert!(event_manager.next_execution().is_none());
    }

    #[test]
    fn test_event_manager_next_box() {
        let event_manager = EventManager::new();

        // insert events
        event_manager
            .emit_priority(GenericEvent, Priority::Interrupt)
            .unwrap();

        let events = event_manager.next_execution().unwrap();
        let events  = events.first().unwrap().1.downcast_ref::<Vec<GenericEvent>>().unwrap();
        

        // assert next execution is interrupt
        assert_eq!(
            *events,
            vec![GenericEvent]
        );
    }
}
