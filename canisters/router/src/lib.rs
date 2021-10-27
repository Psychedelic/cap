use ic_certified_map::{fork, HashTree};
use ic_certified_map::{fork_hash, AsHashTree};
use ic_history_common::canister_list::CanisterList;
use ic_history_common::canister_map::CanisterMap;
use ic_history_common::user_canisters::UserCanisters;
use ic_kit::candid::{candid_method, export_service};
use ic_kit::ic;
use serde::Serialize;

// It's ok.
use ic_cdk::export::Principal;
use ic_history_common::*;
use ic_kit::macros::*;

mod installer;
mod upgrade;

/// Merkle tree of the canister.
///
/// 0: Canister Map
/// 1: User canisters
/// 2: Index canisters list
///
///      ROOT
///     /   \
///   / \    2
///  0   1
#[derive(Serialize)]
pub struct Data {
    /// Map: TokenContractId -> RootBucketId
    root_buckets: CanisterMap,
    /// Map each user to RootBucketId
    user_canisters: UserCanisters,
    /// List of the index canisters.
    index_canisters: CanisterList,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            root_buckets: CanisterMap::default(),
            user_canisters: UserCanisters::default(),
            index_canisters: {
                let mut list = CanisterList::new();
                list.push(ic::id());
                list
            },
        }
    }
}

#[query]
#[candid_method(query)]
fn get_token_contract_root_bucket(
    arg: GetTokenContractRootBucketArg,
) -> GetTokenContractRootBucketResponse {
    let data = ic::get::<Data>();

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    data.root_buckets.gen_witness(&arg.canister),
                    HashTree::Pruned(data.user_canisters.root_hash()),
                ),
                HashTree::Pruned(data.index_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let canister = data.root_buckets.get(&arg.canister).cloned();

    GetTokenContractRootBucketResponse { canister, witness }
}

#[query]
#[candid_method(query)]
fn get_user_root_buckets(arg: GetUserRootBucketsArg) -> GetUserRootBucketsResponse {
    let data = ic::get::<Data>();

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    HashTree::Pruned(data.root_buckets.root_hash()),
                    data.user_canisters.witness(&arg.user),
                ),
                HashTree::Pruned(data.index_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let contracts = data.user_canisters.get(&arg.user).to_vec();

    GetUserRootBucketsResponse { contracts, witness }
}

#[query]
#[candid_method(query)]
fn get_index_canisters(arg: WithWitnessArg) -> GetIndexCanistersResponse {
    let data = ic::get::<Data>();

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                HashTree::Pruned(fork_hash(
                    &data.root_buckets.root_hash(),
                    &data.user_canisters.root_hash(),
                )),
                data.index_canisters.as_hash_tree(),
            )
            .into(),
        ),
    };

    let canisters = data.index_canisters.to_vec();

    GetIndexCanistersResponse { canisters, witness }
}

#[update]
#[candid_method(update)]
fn insert_new_users(contract_id: Principal, users: Vec<Principal>) {
    let data = ic::get_mut::<Data>();
    let root_bucket = ic::caller();

    assert_eq!(
        data.root_buckets.get(&contract_id),
        Some(&root_bucket),
        "Access denied."
    );

    for user in users {
        data.user_canisters.insert(user, root_bucket);
    }
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let dir = dir.parent().unwrap().parent().unwrap().join("candid");
        write(dir.join("router.did"), export_candid()).expect("Write failed.");
    }
}
