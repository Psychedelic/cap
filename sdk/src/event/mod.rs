mod builder;
pub use builder::*;
mod typed;
use cap_sdk_core::transaction::{DetailValue, IndefiniteEvent};
pub use typed::*;

/// Implemented for types that set the `operation` of an
/// event.
pub trait IntoEvent {
    /// The type of operation being executed
    fn operation(&self) -> &'static str {
        ""
    }

    fn details(&self) -> Vec<(String, DetailValue)>;
}

impl IntoEvent for Vec<(String, DetailValue)> {
    fn details(&self) -> Vec<(String, DetailValue)> {
        self.clone()
    }
}

pub trait TryFromEvent: Sized {
    fn try_from_event(event: impl Into<IndefiniteEvent>) -> Result<Self, ()>;
}

impl TryFromEvent for Vec<(String, DetailValue)> {
    fn try_from_event(event: impl Into<IndefiniteEvent>) -> Result<Self, ()> {
        let event = event.into();

        Ok(event.details)
    }
}
