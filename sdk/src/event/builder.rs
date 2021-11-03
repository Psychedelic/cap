use cap_sdk_core::transaction::{DetailValue, EventStatus, IndefiniteEvent};
use ic_kit::Principal;

use super::IntoEvent;

/// Constructs an [`IndefiniteEvent`].
///
/// If the type used for a call to `IndefiniteEventBuilder.details` has a non-default
/// implementation of [`IntoEvent::operation`], it will override the current operation
/// and cause any call to `IndefiniteEventBuilder.operation` to panic.
///
/// # Examples
///
/// ### Create an event with no details
/// Any [`IntoEvent`] type can be substituted into the `details` call.
///
/// ⚠️ Note the above statement if that type uses the non-default
/// implementation of [`IntoEvent::operation`]. ⚠️
///
/// ```rust
//# use cap_sdk_core::transaction::{EventStatus, IndefiniteEvent};
//# use ic_kit::Principal;
//# use crate::IndefiniteEventBuilder;
//# fn wrapper() -> Result<(), ()> {
/// let event = IndefiniteEventBuilder::new()
///     .caller(Principal::anonymous())
///     .status(EventStatus::Completed)
///     .operation("Foo")
///     .details(vec![])
///     .build()?;
//# Ok(())
//# }
///```
#[derive(Default)]
pub struct IndefiniteEventBuilder {
    caller: Option<Principal>,
    status: Option<EventStatus>,
    operation: Option<String>,
    details: Vec<(String, DetailValue)>,
    operation_from_event: bool,
}

impl IndefiniteEventBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the caller of the event.
    pub fn caller(&mut self, caller: Principal) -> &mut Self {
        self.caller = Some(caller);

        self
    }

    /// Sets the status of the event.
    pub fn status(&mut self, status: EventStatus) -> &mut Self {
        self.status = Some(status);

        self
    }

    /// Sets the operation of the event.
    ///
    /// # Panics
    /// Panics if the operation is already set from an [`IntoEvent`] type which
    /// sets the operation.
    pub fn operation(&mut self, operation: impl Into<String>) -> &mut Self {
        if self.operation_from_event {
            panic!("Tried to set operation after it was set from an `IntoEvent` type")
        }

        self.operation = Some(operation.into());

        self
    }

    /// Sets the details of the event.
    ///
    /// You can combine multiple `IntoEvent` types into one [`IndefiniteEvent`]
    /// with this method as long as only one of them has a non-default implementation
    /// of [`IntoEvent::operation`].
    ///
    /// # Panics
    /// Panics if `operation` has already been set by an [`IntoEvent::operation`] call.
    pub fn details(&mut self, details: impl IntoEvent) -> &mut Self {
        self.details.append(&mut details.details());

        if self.operation_from_event {
            panic!("Cannot combine two `IntoEvent` types with unique operations.")
        }

        if details.operation() != "" {
            self.operation_from_event = true;
            self.operation = Some(details.operation().to_owned());
        }

        self
    }

    /// Builds an [`IndefiniteEvent`] from the builder.
    ///
    /// # Panics
    /// Panics if caller, status, operation, or details has not
    /// already been set.
    pub fn build(&mut self) -> Result<IndefiniteEvent, ()> {
        Ok(IndefiniteEvent {
            caller: self.caller.take().unwrap(),
            status: self.status.take().unwrap(),
            operation: self.operation.take().unwrap(),
            details: self.details.clone(),
        })
    }
}
