use cap_common::transaction::{Event, IndefiniteEvent};
use certified_vars::AsHashTree;
use ic_kit::candid::{candid_method, export_service};
use ic_kit::{ic, Principal};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use cap_common::bucket::Bucket;
use cap_common::did::*;
use ic_kit::macros::*;

pub mod upgrade;

/// Merkle tree of the canister.
///
/// 0: Bucket
/// 1: Buckets Lookup Map
/// 2: Next buckets
///
///      ROOT
///     /   \
///   / \    2
///  0   1
#[derive(Serialize, Deserialize)]
pub struct Data {
    pub bucket: Bucket,
    pub users: BTreeSet<Principal>,
    pub cap_id: Principal,
    pub allow_migration: bool,
    pub writers: BTreeSet<TokenContractId>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            bucket: Bucket::new(Principal::management_canister(), 0),
            users: BTreeSet::new(),
            cap_id: Principal::management_canister(),
            allow_migration: true,
            writers: BTreeSet::new(),
        }
    }
}

#[init]
fn init(contract: Principal, writers: BTreeSet<Principal>) {
    let data = ic::get_mut::<Data>();
    data.cap_id = ic::caller();
    data.bucket = Bucket::new(contract, 0);
    data.writers = writers;
}

#[query]
#[candid_method(query)]
fn get_next_canisters(arg: WithWitnessArg) -> GetNextCanistersResponse {
    ic::get::<Data>().bucket.get_next_canisters(arg)
}

#[query]
#[candid_method(query)]
fn get_transaction(arg: WithIdArg) -> GetTransactionResponse {
    ic::get::<Data>().bucket.get_transaction(arg)
}

#[query]
#[candid_method(query)]
fn get_transactions(arg: GetTransactionsArg) -> GetTransactionsResponseBorrowed<'static> {
    ic::get::<Data>().bucket.get_transactions(arg)
}

#[query]
#[candid_method(query)]
fn get_user_transactions(arg: GetUserTransactionsArg) -> GetTransactionsResponseBorrowed<'static> {
    ic::get::<Data>().bucket.get_user_transactions(arg)
}

#[query]
#[candid_method(query)]
fn get_token_transactions(
    arg: GetTokenTransactionsArg,
) -> GetTransactionsResponseBorrowed<'static> {
    ic::get::<Data>().bucket.get_token_transactions(arg)
}

#[query]
#[candid_method(query)]
fn get_bucket_for(arg: WithIdArg) -> GetBucketResponse {
    ic::get::<Data>().bucket.get_bucket_for(arg)
}

#[query]
#[candid_method(query)]
fn time() -> u64 {
    ic::time()
}

#[query]
#[candid_method(query)]
fn size() -> u64 {
    ic::get::<Data>().bucket.size()
}

#[query]
#[candid_method(query)]
fn contract_id() -> &'static Principal {
    ic::get::<Data>().bucket.contract_id()
}

#[update]
#[candid_method(update)]
fn insert(event: IndefiniteEvent) -> TransactionId {
    let data = ic::get_mut::<Data>();
    let caller = ic::caller();

    if !(&caller == data.bucket.contract_id() || data.writers.contains(&caller)) {
        panic!("The method can only be invoked by one of the writers.");
    }

    let event = event.to_event(ic::time() / 1_000_000);

    let mut new_users = Vec::new();
    for principal in event.extract_principal_ids() {
        if data.users.insert(*principal) {
            new_users.push(*principal);
        }
    }

    ic_cdk::block_on(write_new_users_to_cap(
        data.cap_id,
        *data.bucket.contract_id(),
        new_users,
    ));

    let id = data.bucket.insert(event);

    data.allow_migration = false;

    ic::set_certified_data(&data.bucket.root_hash());

    id
}

#[update]
#[candid_method(update)]
fn insert_many(transactions: Vec<IndefiniteEvent>) -> TransactionId {
    let data = ic::get_mut::<Data>();
    let caller = ic::caller();
    let time = ic::time() / 1_000_000;

    if !(&caller == data.bucket.contract_id() || data.writers.contains(&caller)) {
        panic!("The method can only be invoked by one of the writers.");
    }

    let id = data.bucket.size();
    let mut new_users = Vec::new();

    for tx in transactions {
        let event = tx.to_event(time);

        for principal in event.extract_principal_ids() {
            if data.users.insert(*principal) {
                new_users.push(*principal);
            }
        }

        data.bucket.insert(event);
    }

    ic_cdk::block_on(write_new_users_to_cap(
        data.cap_id,
        *data.bucket.contract_id(),
        new_users,
    ));

    ic::set_certified_data(&data.bucket.root_hash());

    id
}

#[update]
#[candid_method(update)]
fn migrate(events: Vec<Event>) {
    let data = ic::get_mut::<Data>();
    let caller = ic::caller();

    if !(&caller == data.bucket.contract_id() || data.writers.contains(&caller)) {
        panic!("The method can only be invoked by one of the writers.");
    }

    if !data.allow_migration {
        panic!("Migration is not allowed after an insert.")
    }

    let mut new_users = Vec::new();

    for event in events {
        for principal in event.extract_principal_ids() {
            if data.users.insert(*principal) {
                new_users.push(*principal);
            }
        }

        data.bucket.insert(event);
    }

    ic_cdk::block_on(write_new_users_to_cap(
        data.cap_id,
        *data.bucket.contract_id(),
        new_users,
    ));

    ic::set_certified_data(&data.bucket.root_hash());
}

async fn write_new_users_to_cap(cap_id: Principal, contract_id: Principal, users: Vec<Principal>) {
    for _ in 0..10 {
        let args = (contract_id, &users);
        if ic::call::<(Principal, &Vec<Principal>), (), &str>(cap_id, "insert_new_users", args)
            .await
            .is_ok()
        {
            break;
        }
    }
}

#[query]
#[candid_method(query)]
fn balance() -> u64 {
    ic::balance()
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[query]
#[candid_method(query)]
fn git_commit_hash() -> String {
    compile_time_run::run_command_str!("git", "rev-parse", "HEAD").into()
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
        write(dir.join("root.did"), export_candid()).expect("Write failed.");
    }
}
