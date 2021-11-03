use std::{convert::TryFrom, path::PrefixComponent};

use cap_sdk_core::transaction::{DetailValue, Event, EventStatus, IndefiniteEvent};
use ic_kit::Principal;

/// A Cap event with typed `details`.
///
/// This type implements [`TryFrom<Event>`]. This allows easy conversion
/// from an [`Event`] into this type, provided the [`Event`]'s details
/// are the correct type. Internally this uses [`TryFromDetails`], which
/// is why this type is bounded on `TryFromDetails + IntoDetails + Sized`.
///
/// Any structures that take [`Event`] use `impl Into<Event>`, so this structure
/// can be used interchangably with [`Event`].
///
/// # Examples
///
/// ### Creating a [`TypedEvent`] from an [`Event`]
///
/// ```rust
//# use std::convert::TryFrom;
//# use cap_sdk_core::transaction::{DetailValue, Event, EventStatus};
//# use ic_kit::Principal;
//# use crate::{IntoDetails, TryFromDetails, TypedEvent};
/// pub struct TransactionDetails {
///     foo: String,
///     bar: u64,
/// }
//#
//# impl TryFromDetails for TransactionDetails {
//#     fn try_from_details(
//#         details: &Vec<(String, cap_sdk_core::transaction::DetailValue)>,
//#     ) -> Result<Self, ()> {
//#         Ok(Self {
//#             foo: String::from(
//#         "foo",
//#        ),
//#        bar: 42,
//#    })
//#}
//#}
//#impl IntoDetails for TransactionDetails {
//#fn into_details(self) -> Vec<(String, cap_sdk_core::transaction::DetailValue)> {
//#    vec![]
//#}
//#}
///
/// // This is an example of the type of event that would be retrieved from
/// // a call to Cap. It has the required details to be cast into a typed
/// // event with our transaction details type.
/// let event = Event {
///     time: 0,
///     caller: Principal::anonymous(),
///     status: EventStatus::Completed,
///     operation: String::from("transfer"),
///     details: vec![
///         ("foo".to_owned(), DetailValue::Text("foo".to_owned())),
///         ("bar".to_owned(), DetailValue::U64(64))
///     ],
/// };
///
/// if let Ok(_typed_event) = TypedEvent::<TransactionDetails>::try_from(event) {
///     // ...
/// } else {
///     panic!("Failed to cast event to typed event.")
/// }
///
/// ```
pub struct TypedEvent<T>
where
    T: TryFromEvent + IntoEvent + Sized,
{
    /// The timestamp in ms.
    pub time: u64,
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The status of the event, can be either `running`, `completed` or `failed`.
    pub status: EventStatus,
    /// The operation that took place.
    pub operation: String,
    /// Details of the transaction.
    pub details: T,
}

impl<T: TryFromEvent + IntoEvent> Into<Event> for TypedEvent<T> {
    fn into(self) -> Event {
        Event {
            time: self.time,
            caller: self.caller,
            status: self.status,
            operation: self.operation,
            details: self.details.details(),
        }
    }
}

impl<T: TryFromEvent + IntoEvent> TryFrom<Event> for TypedEvent<T> {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        Ok(Self {
            time: value.time,
            caller: value.caller,
            status: value.status.clone(),
            operation: value.operation.clone(),
            details: T::try_from_event(value)?,
        })
    }
}

pub struct TypedIndefiniteEvent<T>
where
    T: TryFromEvent + IntoEvent + Sized,
{
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The status of the event, can be either `running`, `completed` or `failed`.
    pub status: EventStatus,
    /// The operation that took place.
    pub operation: String,
    /// Details of the transaction.
    pub details: T,
}

impl<T: TryFromEvent + IntoEvent> Into<IndefiniteEvent> for TypedIndefiniteEvent<T> {
    fn into(self) -> IndefiniteEvent {
        IndefiniteEvent {
            caller: self.caller,
            status: self.status,
            operation: self.operation,
            details: self.details.details(),
        }
    }
}

impl<T: TryFromEvent + IntoEvent> TryFrom<IndefiniteEvent> for TypedIndefiniteEvent<T> {
    type Error = ();

    fn try_from(value: IndefiniteEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            caller: value.caller,
            status: value.status.clone(),
            operation: value.operation.clone(),
            details: T::try_from_event(value)?,
        })
    }
}

/// Constructs an [`IndefiniteEvent`].
#[derive(Default)]
pub struct IndefiniteEventBuilder {
    caller: Option<Principal>,
    status: Option<EventStatus>,
    operation: Option<String>,
    details: Vec<(String, DetailValue)>,
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

    pub fn operation(&mut self, operation: impl Into<String>) -> &mut Self {
        self.operation = Some(operation.into());

        self
    }

    pub fn details(&mut self, details: impl IntoEvent) -> &mut Self {
        self.details.append(&mut details.details());

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
