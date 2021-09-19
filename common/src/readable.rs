use crate::transaction::Event;
use ic_certified_map::{Hash, HashTree};
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

#[derive(Debug, Clone, CandidType, Serialize)]
pub enum GetTransactionResponse<'a> {
    Delegate(ReadableCanisterId, Option<Witness>),
    Found(&'a Event, Option<Witness>),
}

pub type PageKey = [u8; 34];

pub type PageHash = Hash;

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct WithPageArg {
    pub principal: Principal,
    pub page: Option<u32>,
    pub witness: bool,
}

#[derive(Debug, Clone, CandidType, Serialize)]
pub struct GetTransactionsResponse<'a> {
    pub data: Vec<&'a Event>,
    pub page: u32,
    pub witness: Option<Witness>,
}

#[derive(Debug, Clone, Deserialize, CandidType, Serialize)]
pub struct WithWitnessArg {
    pub witness: bool,
}

#[derive(Debug, Clone, CandidType, Serialize)]
pub struct GetCanistersResponse<'a> {
    pub canisters: &'a [ReadableCanisterId],
    pub witness: Option<Witness>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct GetBucketResponse {
    pub canister: ReadableCanisterId,
    pub witness: Option<Witness>,
}

impl Witness {
    #[inline(always)]
    pub fn new(tree: HashTree) -> Self {
        Witness {
            certificate: ic_kit::ic::data_certificate().expect("Data certificate to be present."),
            tree: serde_cbor::to_vec(&tree).unwrap(),
        }
    }
}
