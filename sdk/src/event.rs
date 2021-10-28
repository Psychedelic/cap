use cap_sdk_core::transaction::{EventStatus, IndefiniteEvent};
use ic_kit::Principal;

use crate::IntoDetails;

pub trait IndefiniteEventExt {
    fn from_details(
        caller: Principal,
        status: EventStatus,
        operation: String,
        details: impl IntoDetails,
    ) -> Self;
}

impl IndefiniteEventExt for IndefiniteEvent {
    fn from_details(
        caller: Principal,
        status: EventStatus,
        operation: String,
        details: impl IntoDetails,
    ) -> Self {
        Self {
            caller,
            status,
            operation,
            details: details.into_details(),
        }
    }
}
