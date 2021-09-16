# Internet Computer History OIS Spec

This document goes into the interface description of the Internet Computer History OIS
without implementation. To keep the name short we refer to this service as ICHS in this
document.

The ICHS is a service which provides scalable transaction history that can be used by any
number of tokens or NFTs to store their event log and issue a transaction id to the end
user. It also provides a unified view to the history for every token that integrates this
service for client-facing wallets and network scan UIs.

The primary goal of this project is filling the gap for a native unified ETH-like history
on the Internet Computer.

In this document we go thought the overall architecture of such a service and provide the
schema for the canisters that the system has.

This interface should work regardless of the number of canister the ICHS uses. We define
two groups of canisters. `Readable` and `Writable`. Every canister on the ICHS implements
one or both of these interfaces. For example the entry/router canister implements both
*Readable* and *Writable*

The goal of defining this interfaces is to have implementation agnostic layer of
interaction with this open internet service and future proofing the OIS.

## Readable

The *Readable* interface describes a common schema for performing read-only queries on
the ICHS. We refer to a canister that implements this interface as a Readable Canister.
This interface does not define any operation on the canister that can mutate the state
of the canister.

Not every readable canister is capable of returning the response for the entire data,
and they should be called in the right context. Starting from the entry canister should
always result in valid responses.

## Writable

The *Writable* interface describes an interface for canisters that can mutate the state
of the ICHS. For example inserting a new event to the history.

## Formal Specification

The syntax `cert<K, V>` is an alias for `(blob, blob)`, where the first member of the
tuple is the canister's certificate (the one obtained from `ic0::data_certificate()`),
and the second blob is the cbor serialization of the merkle tree of form
`Tree<Key = K, Value = V>`. See the certified responses section for more information.

```rust
// Information of a transaction. See the for Transaction Type.
use crate::Transaction;

// Principal ID of a writable canister on ICHS.
pub type WritableCanisterId = principal;
// Principal ID of a readable canister on ICHS.
pub type ReadableCanisterId = principal;
// Global transaction ID.
pub type TransactionID = u32;
// Hash of a transaction, 32 bytes.
pub type TransactionHash = blob;

pub enum GetTransactionResponse {
    Found(Option<Transaction>, cert<TransactionID, TransactionHash>),
    Delegate(ReadableCanisterId, cert<TransactionID, ReadableCanisterId>),
}

pub struct GetTransactionsResponse {
    
}

pub trait Readable {
    #[query]
    fn get_transaction(id: TransactionId) -> GetTransactionResponse;
    
    #[query]
    fn get_user_transactions(user: principal, page: u32) -> GetTransactionsResponse;
    
    #[query]
    fn get_token_transactions(user: principal, page: u32) -> GetTransactionsResponse;
}

```

## Transaction Type

## Certified Responses
