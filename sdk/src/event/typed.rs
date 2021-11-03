use std::convert::TryFrom;

use cap_sdk_core::transaction::{Event, EventStatus, IndefiniteEvent};
use ic_kit::Principal;

use super::{IntoEvent, TryFromEvent};

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
    /// Details of the transaction.
    pub details: T,
}

impl<T: IntoEvent + TryFromEvent> TypedEvent<T> {
    /// The operation of the event
    pub fn operation(&self) -> &'static str {
        self.details.operation()
    }
}

impl<T: TryFromEvent + IntoEvent> Into<Event> for TypedEvent<T> {
    fn into(self) -> Event {
        Event {
            time: self.time,
            caller: self.caller,
            status: self.status,
            operation: self.details.operation().to_owned(),
            details: self.details.details(),
        }
    }
}

impl<T: TryFromEvent + IntoEvent> TryFrom<Event> for TypedEvent<T> {
    type Error = T::Error;

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        Ok(Self {
            time: value.time,
            caller: value.caller,
            status: value.status.clone(),
            details: T::try_from_event(value)?,
        })
    }
}

/// A typed indefinite event.
///
/// You can construct an [`IndefiniteEvent`] using a builder, then cast it to
/// a [`TypedIndefiniteEvent`] with [`TryInto`][std::convert::TryInto].
pub struct TypedIndefiniteEvent<T>
where
    T: TryFromEvent + IntoEvent + Sized,
{
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The status of the event, can be either `running`, `completed` or `failed`.
    pub status: EventStatus,
    /// Details of the transaction.
    pub details: T,
}

impl<T: IntoEvent + TryFromEvent> TypedIndefiniteEvent<T> {
    /// The operation of the event.
    pub fn operation(&self) -> &'static str {
        self.details.operation()
    }
}

impl<T: TryFromEvent + IntoEvent> Into<IndefiniteEvent> for TypedIndefiniteEvent<T> {
    fn into(self) -> IndefiniteEvent {
        IndefiniteEvent {
            caller: self.caller,
            status: self.status,
            operation: self.details.operation().to_owned(),
            details: self.details.details(),
        }
    }
}

impl<T: TryFromEvent + IntoEvent> TryFrom<IndefiniteEvent> for TypedIndefiniteEvent<T> {
    type Error = T::Error;

    fn try_from(value: IndefiniteEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            caller: value.caller,
            status: value.status.clone(),
            details: T::try_from_event(value)?,
        })
    }
}
