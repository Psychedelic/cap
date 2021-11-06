use crate::context::CapContext;
use crate::users::Users;
use ic_certified_map::{fork, fork_hash, AsHashTree, HashTree};
use ic_history_common::bucket_lookup_table::BucketLookupTable;
use ic_history_common::canister_list::CanisterList;
use ic_history_common::transaction::{Event, IndefiniteEvent};
use ic_history_common::Bucket;
use ic_kit::candid::{candid_method, export_service};
use ic_kit::{ic, Principal};
use serde::Serialize;
use std::collections::HashSet;

use ic_history_common::did::*;
use ic_kit::macros::*;

mod context;
mod upgrade;
mod users;

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
#[derive(Serialize)]
struct Data {
    bucket: Bucket,
    buckets: BucketLookupTable,
    next_canisters: CanisterList,
    allow_migration: bool,
    users: Users,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            bucket: Bucket::new(0),
            buckets: {
                let mut table = BucketLookupTable::default();
                table.insert(0, ic::id());
                table
            },
            next_canisters: CanisterList::new(),
            allow_migration: true,
            users: Users::default(),
        }
    }
}

#[init]
fn init(contract_id: Principal, writers: HashSet<Principal>) {
    let ctx = CapContext {
        cap_canister_id: ic::caller(),
        contract_id,
        writers,
        ignore_rate_limit: false,
    };

    ic::store(ctx);
}

#[query]
#[candid_method(query)]
fn get_next_canisters(arg: WithWitnessArg) -> GetNextCanistersResponse {
    let data = ic::get::<Data>();

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                HashTree::Pruned(fork_hash(
                    &data.bucket.root_hash(),
                    &data.buckets.root_hash(),
                )),
                data.next_canisters.as_hash_tree(),
            )
            .into(),
        ),
    };

    let canisters = data.next_canisters.to_vec();

    GetNextCanistersResponse { canisters, witness }
}

#[query]
#[candid_method(query)]
fn get_transaction(arg: WithIdArg) -> GetTransactionResponse {
    let data = ic::get::<Data>();

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    data.bucket.witness_transaction(arg.id),
                    HashTree::Pruned(data.buckets.root_hash()),
                ),
                HashTree::Pruned(data.next_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let event = data.bucket.get_transaction(arg.id);

    // We are not multi-canistered yet.
    GetTransactionResponse::Found(event.cloned(), witness)
}

#[query]
#[candid_method(query)]
fn get_transactions(arg: GetTransactionsArg) -> GetTransactionsResponseBorrowed<'static> {
    let data = ic::get::<Data>();
    let ctx = CapContext::get();

    let page = arg
        .page
        .unwrap_or_else(|| data.bucket.last_page_for_contract(&ctx.contract_id));

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    data.bucket
                        .witness_transactions_for_contract(&ctx.contract_id, page),
                    HashTree::Pruned(data.buckets.root_hash()),
                ),
                HashTree::Pruned(data.next_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let events = data
        .bucket
        .get_transactions_for_contract(&ctx.contract_id, page);

    GetTransactionsResponseBorrowed {
        data: events,
        page,
        witness,
    }
}

#[query]
#[candid_method(query)]
fn get_user_transactions(arg: GetUserTransactionsArg) -> GetTransactionsResponseBorrowed<'static> {
    let data = ic::get::<Data>();

    let page = arg
        .page
        .unwrap_or_else(|| data.bucket.last_page_for_user(&arg.user));

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    data.bucket.witness_transactions_for_user(&arg.user, page),
                    HashTree::Pruned(data.buckets.root_hash()),
                ),
                HashTree::Pruned(data.next_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let events = data.bucket.get_transactions_for_user(&arg.user, page);

    GetTransactionsResponseBorrowed {
        data: events,
        page,
        witness,
    }
}

#[query]
#[candid_method(query)]
fn get_bucket_for(arg: WithIdArg) -> GetBucketResponse {
    let data = ic::get::<Data>();

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    HashTree::Pruned(data.bucket.root_hash()),
                    data.buckets.gen_witness(arg.id),
                ),
                HashTree::Pruned(data.next_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let canister = *data.buckets.get_bucket_for(arg.id);

    GetBucketResponse { canister, witness }
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

#[update]
#[candid_method(update)]
fn insert(event: IndefiniteEvent) -> TransactionId {
    let ctx = CapContext::get();
    let caller = ic::caller();

    if !ctx.is_writer(&caller) {
        panic!("The method can only be invoked by one of the writers.");
    }

    let data = ic::get_mut::<Data>();
    let event = event.to_event(ic::time() / 1_000_000);

    let principals = event.extract_principal_ids();
    data.users.insert(ctx, principals);
    data.users.trigger_flush(ctx);

    let id = data.bucket.insert(&ctx.contract_id, event);

    // After the first insertion, prevent the token contract to perform any migration calls.
    data.allow_migration = false;

    ic::set_certified_data(&fork_hash(
        &fork_hash(&data.bucket.root_hash(), &data.buckets.root_hash()),
        &data.next_canisters.root_hash(),
    ));

    id
}

#[update]
#[candid_method(update)]
fn migrate(events: Vec<Event>) {
    let ctx = CapContext::get();
    let caller = ic::caller();

    if !ctx.is_writer(&caller) {
        panic!("The method can only be invoked by one of the writers.");
    }

    let data = ic::get_mut::<Data>();
    if !data.allow_migration {
        panic!("Migration is not allowed after an insert.")
    }

    for event in events {
        let principals = event.extract_principal_ids();
        data.users.insert(ctx, principals);
        data.bucket.insert(&ctx.contract_id, event);
    }

    data.users.trigger_flush(ctx);

    ic::set_certified_data(&fork_hash(
        &fork_hash(&data.bucket.root_hash(), &data.buckets.root_hash()),
        &data.next_canisters.root_hash(),
    ));
}

#[update]
#[candid_method(update)]
fn cap_bypass_rate_limit() {
    let ctx = CapContext::get();
    let caller = ic::caller();

    if ctx.cap_canister_id != caller {
        panic!("This method can only be invoked from cap router.");
    }

    CapContext::bypass_rate_limit();
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
        write(dir.join("root.did"), export_candid()).expect("Write failed.");
    }
}
