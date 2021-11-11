use crate::Data;
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_history_common::bucket_lookup_table::BucketLookupTable;
use ic_history_common::canister_list::CanisterList;
use ic_history_common::transaction::Event;
use ic_history_common::{Bucket, TokenContractId};
use ic_kit::macros::{post_upgrade, pre_upgrade};
use ic_kit::{ic, Principal};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct DataDe {
    bucket: Vec<Event>,
    buckets: BucketLookupTable,
    next_canisters: CanisterList,
    /// List of all the users in this token contract.
    users: BTreeSet<Principal>,
    cap_id: Principal,
    contract: TokenContractId,
    writers: BTreeSet<TokenContractId>,
    allow_migration: bool,
}

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();
    let writer = StableWriter::default();
    serde_cbor::to_writer(writer, &data).expect("Failed to serialize data.");
}

#[post_upgrade]
fn post_upgrade() {
    let reader = StableReader::default();
    let data: DataDe = serde_cbor::from_reader(reader).expect("Failed to deserialize");

    let contract = data.contract;

    let mut bucket = Bucket::new(0);
    for event in data.bucket {
        bucket.insert(&contract, event);
    }

    ic::store(Data {
        bucket,
        buckets: data.buckets,
        next_canisters: data.next_canisters,
        users: data.users,
        cap_id: data.cap_id,
        contract,
        writers: data.writers,
        allow_migration: data.allow_migration,
    })
}
