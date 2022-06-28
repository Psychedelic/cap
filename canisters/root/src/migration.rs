use cap_common::bucket::Bucket;
use cap_common::did::*;
use cap_common::transaction::Event;
use cap_common::TransactionList;
use certified_vars::{Map, Seq};
// use certified_vars::Hash;
use ic_cdk::api::stable::StableReader;
use ic_kit::candid::Principal;
use ic_kit::ic;
use serde::Deserialize;
use std::collections::BTreeSet;

/// f18c9b48287f489ed8c4bac6f0a285b2251a7f4e
pub mod v1 {
    use super::*;

    #[derive(Deserialize)]
    pub struct Data {
        pub bucket: TransactionList,
        pub buckets: Map<TransactionId, Principal>,
        pub next_canisters: Seq<BucketId>,
        pub users: BTreeSet<Principal>,
        pub cap_id: Principal,
        pub contract: TokenContractId,
        pub writers: BTreeSet<TokenContractId>,
        pub allow_migration: bool,
    }

    impl Data {
        pub fn migrate(self) -> crate::Data {
            crate::Data {
                bucket: Bucket::with_transaction_list(self.bucket),
                users: self.users,
                cap_id: self.cap_id,
                allow_migration: self.allow_migration,
                writers: self.writers,
            }
        }
    }
}

/// 9be74b2cf8cf10cd8f9ead09eb44fb3aada01e40
pub mod v0 {
    use super::*;

    #[derive(Deserialize)]
    pub struct CanisterList {
        // commented out because it's not used in the migration, clippy complains
        // data: Vec<Principal>,
        // hash: Hash,
    }

    #[derive(Deserialize)]
    pub struct Data {
        pub bucket: Vec<Event>,
        pub buckets: Vec<(TransactionId, Principal)>,
        pub next_canisters: CanisterList,
        pub users: BTreeSet<Principal>,
        pub cap_id: Principal,
        pub contract: TokenContractId,
        pub writers: BTreeSet<TokenContractId>,
        pub allow_migration: bool,
    }

    impl Data {
        pub fn migrate(self) -> v1::Data {
            let contract = self.contract;

            let mut bucket = TransactionList::new(contract, 0);
            for event in self.bucket {
                bucket.insert(event);
            }

            v1::Data {
                bucket,
                buckets: {
                    let mut table = Map::new();
                    table.insert(0, ic::id());
                    table
                },
                // For now we never had next_canisters,
                // so this is safe.
                next_canisters: Seq::new(),
                users: self.users,
                cap_id: self.cap_id,
                contract,
                writers: self.writers,
                allow_migration: self.allow_migration,
            }
        }
    }
}

pub fn from_stable<T>() -> serde_cbor::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let reader = StableReader::default();
    let mut deserializer = serde_cbor::Deserializer::from_reader(reader);
    let value = serde::de::Deserialize::deserialize(&mut deserializer)?;
    // to allow TrailingData, we comment this line out.
    // deserializer.end()?;
    Ok(value)
}
