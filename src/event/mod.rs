//! # Event Lifecycle
//!
//! Events follow a structured lifecycle. Users emit events into the 
//! `EventManager`, which then queues them for later execution. The `System` periodically 
//! requests events from the `EventManager`, which are then provided in batches to the 
//! event handlers. Once processed, the lifecycle of these events concludes.
//!
//! ## Steps in Event Lifecycle
//!
//! 1. **Event Emission:** Users emit events into the `EventManager`.
//! 2. **Event Queuing:** Emitted events are queued within the `EventManager` for deferred execution.
//! 3. **Event Request:** The `System` requests events from the `EventManager` when ready to process them.
//! 4. **Batch Processing:** The `EventManager` provides events to the `System` in batches, based on the same priority and event type.
//! 5. **Event Handling:** The `System` executes the `Handler` of each event in batch.
//! 6. **Event Completion:** The lifecycle of the processed events concludes.
//!
//! ## Priority Management
//!
//! Events are managed based on priority. The `EventManager` prioritizes events with the 
//! highest priority, ensuring they are executed first. The order of which events are executed
//! is as follows:
//!
//! 1. Interrupt
//! 2. High
//! 3. Normal
//! 4. Routine
//!
//! ## Batch Processing
//! 
//! The `EventManager` provides events to the `System` in batches, based on the same priority and event type.
//! For example, 10 events of type `MyEvent` that has been emitted so far will be grouped into a single batch.
//! batching of events ensures that event processing is efficient.
//! 
//! ## Priority Upgrading
//! 
//! When emitting events of the same type but on different priority say we emit on `Normal` first then on `High`. When such events has not been handled yet,
//! the `EventManager` will promote the event to `High` from `Normal` priority. The priority of such case of events of the same type emitted on different priorities will be upgraded to 
//! the highest priority emitted. 
//! 
//! 
#[doc(hidden)]
pub mod event;
#[doc(inline)]
pub use event::Event;

pub mod priority;

#[doc(hidden)]
pub mod event_manager;
#[doc(inline)]
pub use event_manager::EventManager;
