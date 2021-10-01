use ic_history_common::canister_map::CanisterMap;
use ic_history_common::did::{GetTokenContractRootBucketArg, GetTokenContractRootBucketResponse};
use ic_history_common::{
    GetIndexCanistersResponse, GetUserRootBucketsArg, GetUserRootBucketsResponse, RootBucketId,
    TokenContractId, WithWitnessArg,
};
use ic_kit::ic;
use ic_kit::macros::*;
use ic_kit::Principal;
use std::collections::BTreeMap;

struct Data {
    /// Map: TokenContractId -> RootBucketId
    root_buckets: CanisterMap,
}

#[query]
fn get_token_contract_root_bucket(
    arg: GetTokenContractRootBucketArg,
) -> GetTokenContractRootBucketResponse {
    todo!()
}

#[query]
fn get_user_root_buckets(arg: GetUserRootBucketsArg) -> GetUserRootBucketsResponse {
    todo!()
}

#[query]
fn get_index_canisters(arg: WithWitnessArg) -> GetIndexCanistersResponse {
    todo!()
}

#[update]
fn install_bucket_code(arg: RootBucketId) {
    todo!()
}
