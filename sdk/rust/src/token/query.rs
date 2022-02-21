use cap_sdk_core::Bucket;

use crate::{AsTransactionsPage, CapEnv, GetTransactionsError, GetTransactionsResponse};

/// Gets the transaction with the given token_id [`u64`] 
/// and `page` accepts any [`Into<TransactionsPage>`].
///
/// # Panics
/// Panics if cap is using a multi-canister system, as it
/// is currently unsupported. In this **alpha** release.
///
/// # Examples
/// TODO
pub async fn get_token_transactions(
    token_id: u64,
    page: impl AsTransactionsPage,
) -> Result<GetTransactionsResponse, GetTransactionsError> {
    let context = CapEnv::get().await;

    let as_bucket: Bucket = context.root.into();

    let transactions = as_bucket
        .get_token_transactions(token_id, page.page())
        .await
        .map_err(|(code, details)| GetTransactionsError::Unexpected(code, details))?;

    Ok(GetTransactionsResponse {
        transactions: transactions.data,
        next_page: transactions.page + 1,
    })
}
