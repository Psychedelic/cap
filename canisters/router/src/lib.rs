use ic_certified_map::HashTree::Pruned;
use ic_certified_map::{fork, fork_hash, AsHashTree, Hash, HashTree};
use ic_history_common::bucket_lookup_table::BucketLookupTable;
use ic_history_common::canister_list::CanisterList;
use ic_history_common::readable::{TransactionId, Witness};
use ic_history_common::transaction::IndefiniteEvent;
use ic_history_common::{readable, writable, Bucket};
use ic_kit::ic;
use ic_kit::macros::{query, update};

/// Merkle Tree
///                 /------ READABLE CANISTERS LIST
///         +------+
///        /        \------ BUCKET LOOKUP TABLE
///       /
/// ROOT + --------- BUCKET
struct CanisterStorage {
    bucket: Bucket,
    bucket_lookup_table: BucketLookupTable,
    readable_canisters: CanisterList,
    writable_canisters: Vec<writable::WritableCanisterId>,
}

impl Default for CanisterStorage {
    fn default() -> Self {
        Self {
            bucket: Bucket::new(0),
            bucket_lookup_table: {
                let mut table = BucketLookupTable::default();
                table.insert(0, ic::id());
                table
            },
            readable_canisters: {
                let mut list = CanisterList::default();
                list.push(ic::id());
                list
            },
            writable_canisters: vec![ic::id()],
        }
    }
}

impl AsHashTree for CanisterStorage {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        fork_hash(
            &self.bucket.root_hash(),
            &fork_hash(
                &self.bucket_lookup_table.root_hash(),
                &self.readable_canisters.root_hash(),
            ),
        )
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree {
        fork(
            self.bucket.as_hash_tree(),
            fork(
                self.bucket_lookup_table.as_hash_tree(),
                self.readable_canisters.as_hash_tree(),
            ),
        )
    }
}

#[query]
fn get_index_canisters(
    arg: readable::WithWitnessArg,
) -> readable::GetIndexCanistersResponse<'static> {
    let storage = ic::get::<CanisterStorage>();

    readable::GetIndexCanistersResponse {
        canisters: storage.readable_canisters.as_slice(),
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                Pruned(storage.bucket.root_hash()),
                fork(
                    Pruned(storage.bucket_lookup_table.root_hash()),
                    storage.readable_canisters.as_hash_tree(),
                ),
            ))),
        },
    }
}

#[query]
fn get_bucket_for(arg: readable::WithIdArg) -> readable::GetBucketResponse {
    let storage = ic::get::<CanisterStorage>();

    readable::GetBucketResponse {
        canister: storage.bucket_lookup_table.get_bucket_for(arg.id).clone(),
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                Pruned(storage.bucket.root_hash()),
                fork(
                    storage.bucket_lookup_table.gen_witness(arg.id),
                    Pruned(storage.readable_canisters.root_hash()),
                ),
            ))),
        },
    }
}

#[query]
fn get_transaction(arg: readable::WithIdArg) -> readable::GetTransactionResponse<'static> {
    let storage = ic::get::<CanisterStorage>();
    let transaction = storage.bucket.get_transaction(arg.id);

    if let Some(event) = transaction {
        readable::GetTransactionResponse::Found(
            event,
            match arg.witness {
                false => None,
                true => Some(Witness::new(fork(
                    storage.bucket.witness_transaction(arg.id),
                    Pruned(fork_hash(
                        &storage.bucket_lookup_table.root_hash(),
                        &storage.readable_canisters.root_hash(),
                    )),
                ))),
            },
        )
    } else {
        let r = get_bucket_for(arg);
        readable::GetTransactionResponse::Delegate(r.canister, r.witness)
    }
}

#[query]
fn get_user_transactions(arg: readable::WithPageArg) -> readable::GetTransactionsResponse<'static> {
    let storage = ic::get::<CanisterStorage>();
    let page = storage
        .bucket
        .get_transactions_for_user(&arg.principal, arg.page);

    readable::GetTransactionsResponse {
        data: page,
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                storage
                    .bucket
                    .witness_transactions_for_user(&arg.principal, arg.page),
                Pruned(fork_hash(
                    &storage.bucket_lookup_table.root_hash(),
                    &storage.readable_canisters.root_hash(),
                )),
            ))),
        },
    }
}

#[query]
fn get_token_transactions(
    arg: readable::WithPageArg,
) -> readable::GetTransactionsResponse<'static> {
    let storage = ic::get::<CanisterStorage>();
    let page = storage
        .bucket
        .get_transactions_for_token(&arg.principal, arg.page);

    readable::GetTransactionsResponse {
        data: page,
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                storage
                    .bucket
                    .witness_transactions_for_token(&arg.principal, arg.page),
                Pruned(fork_hash(
                    &storage.bucket_lookup_table.root_hash(),
                    &storage.readable_canisters.root_hash(),
                )),
            ))),
        },
    }
}

#[update]
fn get_writer_canisters() -> &'static [writable::WritableCanisterId] {
    let storage = ic::get::<CanisterStorage>();
    storage.writable_canisters.as_slice()
}

#[update]
fn insert(event: IndefiniteEvent) -> TransactionId {
    let storage = ic::get_mut::<CanisterStorage>();
    let event = event.to_event(ic::caller(), ic::time() / 1_000_000);
    let id = storage.bucket.insert(event);
    ic::set_certified_data(&storage.root_hash());
    id
}

#[query]
fn time() -> u64 {
    ic::time()
}
