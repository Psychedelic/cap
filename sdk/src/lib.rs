#![allow(clippy::from_over_into)]

//! The SDK for integrating Certified Asset Provenance (CAP) into an Internet Computer canister.
//!
//!

pub use env::*;
mod env;

mod transactions;
pub use transactions::*;

mod transaction;
pub use transaction::*;

mod insert;
pub use insert::*;

mod details;
pub use details::*;

mod event;
pub use event::*;

use cap_sdk_core::transaction::Event;

type Transaction = Event;
type TransactionId = u64;

#[doc(hidden)]
mod prelude {
    pub use crate::*;
}

pub use prelude::*;
