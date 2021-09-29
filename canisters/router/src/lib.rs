use ic_history_common::{RootBucketId, TokenContractId};
use ic_kit::ic;
use ic_kit::macros::*;
use ic_kit::Principal;
use std::collections::BTreeMap;

struct Data {
    root_buckets: BTreeMap<TokenContractId, RootBucketId>,
}

#[query]
fn get_token_contract_root_bucket() {}
