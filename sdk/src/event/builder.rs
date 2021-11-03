use cap_sdk_core::transaction::{DetailValue, EventStatus, IndefiniteEvent};
use ic_kit::Principal;

use super::IntoEvent;

/// Constructs an [`IndefiniteEvent`].
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

    pub fn caller(&mut self, caller: Principal) -> &mut Self {
        self.caller = Some(caller);

        self
    }

    pub fn status(&mut self, status: EventStatus) -> &mut Self {
        self.status = Some(status);

        self
    }

    ///
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

    pub fn details(&mut self, details: impl IntoEvent) -> &mut Self {
        self.details.append(&mut details.details());

        if details.operation() != "" {
            self.operation_from_event = true;
            self.operation = Some(details.operation().to_owned());
        }

        self
    }

    pub fn build(&mut self) -> Result<IndefiniteEvent, ()> {
        Ok(IndefiniteEvent {
            caller: self.caller.take().unwrap(),
            status: self.status.take().unwrap(),
            operation: self.operation.take().unwrap(),
            details: self.details.clone(),
        })
    }
}
