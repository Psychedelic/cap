use cap_sdk_core::GetTransactionResponse;

use crate::{CapEnv, GetTransactionError, Transaction, TransactionId};

/// Gets the transaction with the given id.
///
/// # Panics
/// Panics if cap is using a multi-canister system, as it
/// is currently unsupported. In this **alpha** release.
///
/// # Examples
/// TODO
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
