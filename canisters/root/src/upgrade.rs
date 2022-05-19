use crate::Data;
use cap_common::bucket::Bucket;
use cap_common::{BucketId, TokenContractId, TransactionId, TransactionList};
use certified_vars::{Map, Seq};
use ic_cdk::api::stable::StableReader;
use ic_kit::macros::{post_upgrade, pre_upgrade};
use ic_kit::{ic, Principal};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::Read;

#[derive(Deserialize)]
struct DataV0 {
    bucket: TransactionList,
    buckets: Map<TransactionId, Principal>,
    next_canisters: Seq<BucketId>,
    /// List of all the users in this token contract.
    users: BTreeSet<Principal>,
    cap_id: Principal,
    contract: TokenContractId,
    writers: BTreeSet<TokenContractId>,
    allow_migration: bool,
}

#[pre_upgrade]
fn pre_upgrade() {
    ic::stable_store((ic::get::<Data>(),)).expect("Failed to serialize data.");
}

#[post_upgrade]
pub fn post_upgrade() {
    let reader = StableReader::default();

    let data: DataV0 = match serde_cbor::from_reader(reader) {
        Ok(t) => t,
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).expect("Failed to deserialize.")
        }
    };

    ic::store(Data {
        bucket: Bucket::with_transaction_list(data.bucket),
        users: data.users,
        cap_id: data.cap_id,
        allow_migration: data.allow_migration,
        writers: data.writers,
    });
}
