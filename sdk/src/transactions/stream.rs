use async_stream::try_stream;
use cap_sdk_core::Bucket;
use futures::Stream;

use crate::{CapEnv, GetTransactionsError, Transaction};

pub async fn get_transactions(
    start_page: u32,
    end_page: u32,
) -> impl Stream<Item = Result<Transaction, GetTransactionsError>> {
    try_stream! {
        let context = CapEnv::get();

        let bucket: Bucket = context.root.into();

        for page in (start_page..end_page) {
            let transactions = bucket
                .get_transactions(Some(page), false)
                .await
                .map_err(|(code, details)| GetTransactionsError::Unexpected(code, details))?;

            for transaction in transactions.data {
                yield transaction
            }
        }

    }
}
