use ic_history_common::transaction::IndefiniteEvent;
use ic_history_common::Bucket;
use ic_kit::{ic, Principal};
use std::collections::BTreeSet;

use ic_certified_map::{fork, fork_hash, AsHashTree, HashTree};
use ic_history_common::bucket_lookup_table::BucketLookupTable;
use ic_history_common::canister_list::CanisterList;
use ic_history_common::did::*;
use ic_kit::macros::*;

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
struct Data {
    bucket: Bucket,
    buckets: BucketLookupTable,
    next_canisters: CanisterList,
    contract: TokenContractId,
    writers: BTreeSet<TokenContractId>,
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
            next_canisters: CanisterList::default(),
            contract: Principal::management_canister(),
            writers: BTreeSet::new(),
        }
    }
}

#[query]
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
fn get_transactions(arg: GetTransactionsArg) -> GetTransactionsResponseBorrowed<'static> {
    let data = ic::get::<Data>();

    let page = arg
        .page
        .unwrap_or(data.bucket.last_page_for_contract(&data.contract));

    let witness = match arg.witness {
        false => None,
        true => Some(
            fork(
                fork(
                    data.bucket
                        .witness_transactions_for_contract(&data.contract, page),
                    HashTree::Pruned(data.buckets.root_hash()),
                ),
                HashTree::Pruned(data.next_canisters.root_hash()),
            )
            .into(),
        ),
    };

    let events = data
        .bucket
        .get_transactions_for_contract(&data.contract, page);

    GetTransactionsResponseBorrowed {
        data: events,
        page,
        witness,
    }
}

#[query]
fn get_user_transactions(arg: GetUserTransactionsArg) -> GetTransactionsResponseBorrowed<'static> {
    let data = ic::get::<Data>();

    let page = arg
        .page
        .unwrap_or(data.bucket.last_page_for_user(&arg.user));

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

    let canister = data.buckets.get_bucket_for(arg.id).clone();

    GetBucketResponse { canister, witness }
}

#[query]
fn time() -> u64 {
    ic::time()
}

#[update]
fn insert(event: IndefiniteEvent) -> TransactionId {
    let data = ic::get_mut::<Data>();
    let caller = ic::caller();

    if !(&caller == &data.contract || data.writers.contains(&caller)) {
        panic!("The method can only be invoked by one of the writers.");
    }

    let event = event.to_event(ic::time() / 1_000_000);
    let id = data.bucket.insert(&data.contract, event);

    ic::set_certified_data(&fork_hash(
        &fork_hash(&data.bucket.root_hash(), &data.buckets.root_hash()),
        &data.next_canisters.root_hash(),
    ));

    id
}
