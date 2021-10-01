//! This file contains all of the type definitions used in the candid
//! files across the different canisters and the services.

use ic_certified_map::Hash;
use ic_kit::candid::{CandidType, Deserialize};
use ic_kit::Principal;
use serde::Serialize;

/// The numeric type used to represent a transaction id.
pub type TransactionId = u64;

/// The principal id of a index canister.
pub type IndexCanisterId = Principal;

/// The principal id of a root bucket canister.
pub type RootBucketId = Principal;

/// The principal id of a bucket canister.
pub type BucketId = Principal;

/// The principal id of a token contract this is integrating Cap.
pub type TokenContractId = Principal;

/// The principal id of a user.
pub type UserId = Principal;

/// Hash of an even which is obtained by `Event::hash`
pub type EventHash = Hash;

#[derive(Serialize, Deserialize, CandidType)]
pub struct Witness {
    #[serde(with = "serde_bytes")]
    certificate: Vec<u8>,
    #[serde(with = "serde_bytes")]
    tree: Vec<u8>,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct GetTokenContractRootBucketArg {
    pub canister: TokenContractId,
    pub witness: bool,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct GetTokenContractRootBucketResponse {
    pub canister: Option<RootBucketId>,
    pub witness: Option<Witness>,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct GetUserRootBucketsArg {
    pub user: UserId,
    pub witness: bool,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct GetUserRootBucketsResponse {
    pub contracts: Vec<RootBucketId>,
    pub witness: Option<Witness>,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct WithWitnessArg {
    pub witness: bool,
}

#[derive(Serialize, Deserialize, CandidType)]
pub struct GetIndexCanistersResponse {
    pub canisters: Vec<IndexCanisterId>,
    pub witness: Option<Witness>,
}
