use crate::env::CapEnv;
use cap_core::{
    transaction::{Event, IndefiniteEvent},
    Bucket, GetTransactionResponse,
};
use futures::Stream;
use ic_kit::{Principal, RejectionCode};
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

mod env;

type Transaction = Event;
type TransactionId = u64;

pub struct GetTransactionsResponse {
    pub data: Vec<()>,
    // next_page_context: Option<NextPageContext>
}

/// Gets the transaction with the given id.
pub async fn get_transaction(id: TransactionId) -> Result<Transaction, GetTransactionError> {
    let context = CapEnv::get();

    let bucket = context
        .root
        .get_bucket_for(id)
        .await
        .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;

    let transaction = bucket
        .get_transaction(id, false)
        .await
        .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;

    if let GetTransactionResponse::Found(event, _) = transaction {
        if let Some(event) = event {
            return Ok(event);
        } else {
            return Err(GetTransactionError::InvalidId);
        }
    } else {
        // TODO: Delegate
        unimplemented!()
    }
}

/// An error thrown when querying for a contract's root bucket.
#[derive(Error, Debug)]
pub enum GetTransactionError {
    /// The bucket rejected the call for an unexpected reason.
    #[error("the query was rejected")]
    Unexpected(RejectionCode, String),
    #[error("no transaction found with the given id")]
    InvalidId,
}

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

/// An error thrown when querying for a contract's root bucket.
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

/// Gets the transactions on the given page for the contract.
///
/// Returns all transactions on the page, and the next page.
pub async fn get_transactions(
    page: Option<u32>,
) -> Result<(Vec<Transaction>, u32), GetTransactionError> {
    let context = CapEnv::get();

    let bucket: Bucket = context.root.into();

    let transactions = bucket
        .get_transactions(page, false)
        .await
        .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;

    Ok((transactions.data, transactions.page + 1))
}
