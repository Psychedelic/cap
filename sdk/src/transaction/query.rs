use cap_sdk_core::{Bucket, GetTransactionResponse};

use crate::{CapEnv, GetTransactionError, Transaction, TransactionId};

/// Gets the transaction with the given id.
///
/// # Panics
/// Panics if cap is using a multi-canister system, as it
/// is currently unsupported. In this **alpha** release.
///
/// # Examples
/// ### Query an event and use [`TypedEvent`] to make it easy to work with.
///
/// ```rust
/// // Retrieve a transaction from cap. Since this contract uses the
/// // DIP20 standard we know its DIP20 compliant and will unwrap the
/// // conversion.
/// let transaction = get_transaction(230948).unwrap();
///
/// let typed_transaction: TypedEvent<DIP20Details> = transaction.try_into().unwrap();
/// ```
pub async fn get_transaction(id: TransactionId) -> Result<Transaction, GetTransactionError> {
    let context = CapEnv::get().await;

    let bucket = context
        .root
        .get_bucket_for(id)
        .await
        .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;

    let mut transaction = bucket
        .get_transaction(id)
        .await
        .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;

    while let GetTransactionResponse::Delegate(bucket, _) = transaction {
        transaction = Bucket(bucket)
            .get_transaction(id)
            .await
            .map_err(|(code, details)| GetTransactionError::Unexpected(code, details))?;
    }

    if let GetTransactionResponse::Found(Some(event), _) = transaction {
        Ok(event)
    } else {
        Err(GetTransactionError::InvalidId)
    }
}
