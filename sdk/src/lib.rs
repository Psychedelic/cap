use std::pin::Pin;
use cap_core::GetTransactionResponse;
use futures::Stream;
use std::task::{Context, Poll};
use ic_kit::Principal;
use crate::env::CapEnv;

mod env;

type Transaction = ();
type TransactionId = u64;

pub struct GetTransactionsResponse {
    pub data: Vec<()>,
    // next_page_context: Option<NextPageContext>
}

pub struct GetTransactionsStream {}

impl GetTransactionsStream {
    async fn new() -> Self {
        let context = CapEnv::get().await;

        todo!()
    }
}

impl Stream for GetTransactionsStream {
    type Item = Transaction;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO: Figure out if we can do this
        (0, None)
    }
}


pub async fn get_transaction(id: TransactionId) -> Result<Transaction, GetTransactionError> {
    let context = CapEnv::get().await;

    // Impl From<GetTransactionWhatever>
    let bucket = context.root.get_bucket_for(id).await?;

    let transaction = bucket.get_transaction(id, false).await?;

    if let GetTransactionResponse::Found(_, _) = transaction {} else {
        unimplemented!()
    }

    todo!()
}

pub enum GetTransactionError {}