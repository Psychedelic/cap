use cap_sdk_core::transaction::IndefiniteEvent;
use std::cell::RefCell;

use crate::{env::CapEnv, InsertTransactionError, TransactionId};

thread_local! {
    pub(crate) static PENDING: RefCell<Vec<IndefiniteEvent>> = RefCell::new(vec![]);
}

/// Inserts a transaction into the contract's history.
///
/// # Examples
///
/// ### Inserting an event from a builder.
///
/// See also: [`IndefiniteEventBuilder`][crate::IndefiniteEventBuilder], [`IntoDetails`][crate::IntoDetails], [`TryFromDetails`][crate::TryFromDetails].
/// ```rust
/// use cap_sdk_core::transaction::EventStatus;
//# use ic_kit::Principal;
//# use crate::{insert, IndefiniteEventBuilder, IntoDetails, TryFromDetails};
/// pub struct TransactionDetails {
///     foo: String,
///     bar: u64,
/// }
//# impl TryFromDetails for TransactionDetails {
//# fn try_from_details(
//#    details: &Vec<(String, cap_sdk_core::transaction::DetailValue)>,
//# ) -> Result<Self, ()> {
//#    Ok(Self {
//#        foo: String::from(
//#            "Peek behind the curtain and you might regret what you find...",
//#        ),
//#        bar: 42,
//#    })
//# }
//# }
//# impl IntoDetails for TransactionDetails {
//# fn into_details(self) -> Vec<(String, cap_sdk_core::transaction::DetailValue)> {
//#     vec![]
//# }
//# }
///
/// let transaction_details = TransactionDetails {
///     foo: String::from("foo"),
///     bar: 42
/// };
///
/// // Construct the event which accompanies our details.
/// // `IndefiniteEventBuilder` allows any `IntoDetails` type
/// // to be used in a call to `IndefiniteEventBuilder::details`.
/// //
/// // This is useful when dealing with the `cap-standards` types,
/// // but also works with a dynamically-constructed details vec with
/// // the type signature of `Vec<(String, DetailValue)>`.
/// let event = IndefiniteEventBuilder::new()
///     .caller(Principal::anonymous())
///     .operation(String::from("transfer"))
///     .status(EventStatus::Completed)
///     .details(transaction_details)
///     .build()
///     .unwrap();
///
///
/// // Insert the transaction with `insert`. It takes any type
/// // that implements `Into<IndefiniteEvent>`, this includes
/// // types like `TypedIndefiniteEvent` as well as `Vec<(String, DetailValue)>`
/// insert(event).await.unwrap();
/// ```
pub async fn insert(
    transaction: impl Into<IndefiniteEvent>,
) -> Result<TransactionId, InsertTransactionError> {
    let context = CapEnv::get().await;
    let event = transaction.into();

    let id = context
        .root
        .insert(&event)
        .await
        .map_err(|(code, details)| match details.as_str() {
            "The method can only be invoked by one of the writers." => {
                InsertTransactionError::CantWrite
            }
            _ => InsertTransactionError::Unexpected(code, details),
        })?;

    Ok(id)
}

pub fn insert_sync(event: impl Into<IndefiniteEvent>) {
    PENDING.with(|p| {
        p.borrow_mut().push(event.into());
    });

    ic_cdk::block_on(async {
        let _ = perform_insert().await;
    });
}

pub fn insert_many_sync<T: Into<IndefiniteEvent>>(events: impl Iterator<Item = T>) {
    PENDING.with(|p| {
        p.borrow_mut().extend(events.map(|e| e.into()));
    });

    ic_cdk::block_on(async {
        let _ = perform_insert().await;
    });
}

pub async fn perform_insert() -> Result<(), InsertTransactionError> {
    let context = CapEnv::get().await;
    let mut events = PENDING.with(|p| p.take());

    context
        .root
        .insert_many(&events)
        .await
        .map_err(|(code, details)| match details.as_str() {
            "The method can only be invoked by one of the writers." => {
                InsertTransactionError::CantWrite
            }
            _ => InsertTransactionError::Unexpected(code, details),
        })
        .map_err(|e| {
            // TODO(qti3e) Is ordering preserved this way?
            // need to be double checked.
            PENDING.with(|p| {
                events.extend(p.take());
                p.replace(events);
            });

            e
        })?;

    Ok(())
}
