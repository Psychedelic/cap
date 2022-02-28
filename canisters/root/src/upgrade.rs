use crate::Data;
use cap_common::transaction::Event;
use cap_common::{Bucket, TokenContractId, TransactionId};
use certified_vars::{Hash, Map, Seq};
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_kit::macros::{post_upgrade, pre_upgrade};
use ic_kit::{ic, Principal};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::Read;

#[derive(Deserialize)]
struct DataV0 {
    bucket: Vec<Event>,
    _buckets: Vec<(TransactionId, Principal)>,
    _next_canisters: CanisterListV0,
    /// List of all the users in this token contract.
    users: BTreeSet<Principal>,
    cap_id: Principal,
    contract: TokenContractId,
    writers: BTreeSet<TokenContractId>,
    allow_migration: bool,
}

#[derive(Deserialize)]
pub struct CanisterListV0 {
    _data: Vec<Principal>,
    _hash: Hash,
}

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();
    let writer = StableWriter::default();
    serde_cbor::to_writer(writer, &data).expect("Failed to serialize data.");
}

pub fn next_post_upgrade() {
    let reader = StableReader::default();
    let data: Data = match serde_cbor::from_reader(reader) {
        Ok(t) => t,
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).expect("Failed to deserialize.")
        }
    };
    ic::store(data);
}

#[post_upgrade]
fn post_upgrade() {
    let reader = StableReader::default();
    let data: DataV0 = match serde_cbor::from_reader(reader) {
        Ok(t) => t,
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).expect("Failed to deserialize.")
        }
    };

    let contract = data.contract;

    let mut bucket = Bucket::new(contract, 0);
    for event in data.bucket {
        bucket.insert(event);
    }

    ic::store(Data {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap_common::transaction::DetailValue;
    use ic_kit::{MockContext, Principal};

    const fn p(id: u8) -> Principal {
        Principal::from_slice(&[id, 0x00])
    }

    #[test]
    fn test() {
        let root_bucket_id = p(0);
        let contract_id = p(1);

        MockContext::new()
            .with_id(root_bucket_id)
            .with_caller(contract_id)
            .inject();

        let data = ic::get_mut::<Data>();

        for i in 0..100 {
            let e = Event {
                time: i as u64,
                caller: p(i + 5),
                operation: "mint".to_string(),
                details: vec![("amount".into(), DetailValue::U64(i as u64))],
            };
            data.bucket.insert(e);
        }

        let serialized = serde_cbor::to_vec(data).expect("Failed to serialize.");
        let actual: Data = serde_cbor::from_slice(&serialized).expect("Failed to deserialize.");

        assert_eq!(actual.bucket.len(), 100);
    }
}
