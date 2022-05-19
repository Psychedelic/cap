use crate::Data;
use cap_common::bucket::Bucket;
use cap_common::transaction::Event;
use cap_common::{BucketId, TokenContractId, TransactionId, TransactionList};
use certified_vars::{Hash, Map, Seq};
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

#[derive(Deserialize)]
struct DataV00 {
    bucket: Vec<Event>,
    _buckets: Vec<(TransactionId, Principal)>,
    _next_canisters: CanisterListV00,
    /// List of all the users in this token contract.
    users: BTreeSet<Principal>,
    cap_id: Principal,
    contract: TokenContractId,
    writers: BTreeSet<TokenContractId>,
    allow_migration: bool,
}

#[derive(Deserialize)]
pub struct CanisterListV00 {
    _data: Vec<Principal>,
    _hash: Hash,
}

#[pre_upgrade]
fn pre_upgrade() {
    ic::stable_store((ic::get::<Data>(),)).expect("Failed to serialize data.");
}

#[post_upgrade]
pub fn post_upgrade() {
    let reader = StableReader::default();

    let data: Option<DataV00> = match serde_cbor::from_reader(reader) {
        Ok(t) => Some(t),
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).ok()
        }
    };

    let data = if let Some(data) = data {
        let contract = data.contract;

        let mut bucket = TransactionList::new(contract, 0);
        for event in data.bucket {
            bucket.insert(event);
        }

        DataV0 {
            bucket,
            buckets: {
                let mut table = Map::new();
                table.insert(0, ic::id());
                table
            },
            // For now we never had next_canisters,
            // so this is safe.
            next_canisters: Seq::new(),
            users: data.users,
            cap_id: data.cap_id,
            contract,
            writers: data.writers,
            allow_migration: data.allow_migration,
        }
    } else {
        let reader = StableReader::default();
        let data: DataV0 = match serde_cbor::from_reader(reader) {
            Ok(t) => t,
            Err(err) => {
                let limit = err.offset() - 1;
                let reader = StableReader::default().take(limit);
                match serde_cbor::from_reader(reader) {
                    Ok(e) => e,
                    Err(e) => ic::trap(&e.to_string()),
                }
            }
        };
        data
    };

    ic::store(Data {
        bucket: Bucket::with_transaction_list(data.bucket),
        users: data.users,
        cap_id: data.cap_id,
        allow_migration: data.allow_migration,
        writers: data.writers,
    });
}
