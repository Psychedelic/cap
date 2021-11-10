use cap_sdk_core::{Bucket, GetTransactionResponse};
use ic_kit::Principal;

use crate::{AsTransactionsPage, CapEnv, GetTransactionError, Transaction};

/// Gets the transaction with the given id.
///
/// # Panics
/// Panics if cap is using a multi-canister system, as it
/// is currently unsupported. In this **alpha** release.
///
/// # Examples
/// TODO
pub async fn get_user_transaction(
    user: Principal,
    page: impl AsTransactionsPage,
) -> Result<Transaction, GetTransactionError> {
    let context = CapEnv::get();

    let as_bucket: Bucket = context.root.into();

    let transaction = as_bucket
        .get_user_transactions(user, page.page())
        .await
        .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;

    if let GetTransactionResponse::Found(event, _) = transaction {
        if let Some(event) = event {
            Ok(event)
        } else {
            Err(GetTransactionError::InvalidId)
        }
    } else {
        // TODO: Delegate
        unimplemented!("This version of cap-sdk does not support multi-canister.")
    }
}
