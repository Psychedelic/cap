use cap_sdk_core::Bucket;
use ic_kit::RejectionCode;
use thiserror::Error;

use crate::{env::CapEnv, Transaction};

/// The response given from a [`get_transactions`] call.
#[derive(Debug, Clone)]
pub struct GetTransactionsResponse {
    transactions: Vec<Transaction>,
    next_page: u32,
}

impl GetTransactionsResponse {
    /// Returns the transactions associated with this response.
    #[inline(always)]
    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    /// Returns the next page number.
    #[inline(always)]
    pub fn next_page(&self) -> u32 {
        self.next_page
    }

    /// Converts a [`GetTransactionsResponse`] to the transactions within it.
    #[inline(always)]
    pub fn into_transactions(self) -> Vec<Transaction> {
        self.into()
    }
}

impl Into<Vec<Transaction>> for GetTransactionsResponse {
    fn into(self) -> Vec<Transaction> {
        self.transactions
    }
}

/// A type that represents a page of a transaction.
#[derive(Debug, Clone, Copy)]
pub struct TransactionsPage(pub Option<u32>);

impl Into<TransactionsPage> for Option<u32> {
    fn into(self) -> TransactionsPage {
        TransactionsPage(self)
    }
}

impl Into<TransactionsPage> for &GetTransactionsResponse {
    fn into(self) -> TransactionsPage {
        TransactionsPage(Some(self.next_page))
    }
}

#[derive(Error, Debug)]
pub enum GetTransactionsError {
    /// The bucket rejected the call for an unexpected reason.
    #[error("the query was rejected")]
    Unexpected(RejectionCode, String),
    #[error("no transaction found with the given id")]
    InvalidId,
}

/// Gets a transaction for the given page.
///
/// `page` accepts any [`Into<TransactionsPage>`].
///
/// This is implemented for [`Option<u32>`] and &[`GetTransactionsResponse`].
///
/// This allows you to query for the next page from a response, as well as
/// any given page.
///
/// # Panics
/// Panics if cap is using a multi-canister system, as it
/// is currently unsupported. In this **alpha** release.
///
/// # Examples
/// TODO
pub async fn get_transactions(
    page: impl Into<TransactionsPage>,
) -> Result<GetTransactionsResponse, GetTransactionsError> {
    let context = CapEnv::get();

    let bucket: Bucket = context.root.into();

    let transactions = bucket
        .get_transactions(page.into().0, false)
        .await
        .map_err(|(code, details)| GetTransactionsError::Unexpected(code, details))?;

    Ok(GetTransactionsResponse {
        transactions: transactions.data,
        next_page: transactions.page + 1,
    })
}
