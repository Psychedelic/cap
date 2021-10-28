use cap_sdk_core::transaction::IndefiniteEvent;

use ic_kit::RejectionCode;
use thiserror::Error;

use crate::{env::CapEnv, TransactionId};

/// Inserts a transaction into the contract's history.
///
/// # Examples
/// TODO
pub async fn insert(transaction: IndefiniteEvent) -> Result<TransactionId, InsertTransactionError> {
    let context = CapEnv::get();

    let id = context
        .root
        .insert(transaction)
        .await
        .map_err(|(code, details)| match details.as_str() {
            "The method can only be invoked by one of the writers." => {
                InsertTransactionError::CantWrite
            }
            _ => InsertTransactionError::Unexpected(code, details),
        })?;

    Ok(id)
}

#[derive(Error, Debug)]
pub enum InsertTransactionError {
    /// The bucket rejected the call for an unexpected reason.
    #[error("the query was rejected")]
    Unexpected(RejectionCode, String),
    /// Returned when `insert` is called on a root canister that
    /// does not accept writes from the calling canister.
    #[error("the root canister does not accept writes from this canister")]
    CantWrite,
    #[error("no transaction found with the given id")]
    InvalidId,
}
