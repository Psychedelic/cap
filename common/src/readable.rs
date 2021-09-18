use crate::transaction::Event;
use ic_certified_map::Hash;
use ic_kit::candid::{CandidType, Deserialize};
use ic_kit::Principal;
use serde::Serialize;

/// Principal ID of a readable canister.
pub type ReadableCanisterId = Principal;

/// The witness returned by the query methods.
#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub struct Witness {
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub tree: Vec<u8>,
}

/// Hash of an event. Obtained from calling Event::hash().
pub type EventHash = Hash;

/// The ID of a transaction.
pub type TransactionId = u64;

#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub struct WithIdArg {
    pub id: TransactionId,
    pub witness: bool,
}

#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub enum GetTransactionResponse {
    Delegate(ReadableCanisterId, Option<Witness>),
    Found(Event, Option<Witness>),
}

pub type PageKey = [u8; 34];

pub type PageHash = Hash;

pub struct WithPageArg {
    pub principal: Principal,
    pub page: u32,
    pub witness: bool,
}

#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub enum GetTransactionsResponse {
    Delegate(ReadableCanisterId, Option<Witness>),
    Found(Vec<Event>, Option<Witness>),
}

#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub struct WithWitnessArg {
    pub witness: bool,
}

#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub struct GetIndexCanistersResponse {
    pub canisters: Vec<ReadableCanisterId>,
    pub witness: Option<Witness>,
}

pub struct GetBucketResponse {
    pub canister: ReadableCanisterId,
    pub witness: Option<Witness>,
}
