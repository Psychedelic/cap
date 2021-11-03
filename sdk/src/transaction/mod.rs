use ic_kit::RejectionCode;
use thiserror::Error;

mod insert;
pub use insert::insert;

mod query;
pub use query::get_transaction;

#[derive(Error, Debug)]
pub enum GetTransactionError {
    /// The bucket rejected the call for an unexpected reason.
    #[error("the query was rejected")]
    Unexpected(RejectionCode, String),
    #[error("no transaction found with the given id")]
    InvalidId,
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
