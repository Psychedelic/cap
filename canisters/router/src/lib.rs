use ic_certified_map::HashTree::Pruned;
use ic_certified_map::{fork, fork_hash, AsHashTree, Hash, HashTree};
use ic_history_common::bucket_lookup_table::BucketLookupTable;
use ic_history_common::canister_list::CanisterList;
use ic_history_common::readable::{TransactionId, Witness};
use ic_history_common::transaction::IndefiniteEvent;
use ic_history_common::{readable, writable, Bucket};
use ic_kit::ic;
use ic_kit::macros::{query, update};

/// Tree
///
/// 0: Bucket
/// 1: Bucket Lookup Table
/// 3: Readable Canisters
/// 4: Next Canisters
///
/// ```text
///       ROOT
///      /    \
///     /      \
///    V        V
///   /  \     /  \
///  0    1   3    4
/// ```
struct CanisterStorage {
    bucket: Bucket,
    bucket_lookup_table: BucketLookupTable,
    readable_canisters: CanisterList,
    next_canisters: CanisterList,
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
            next_canisters: CanisterList::default(),
            writable_canisters: vec![ic::id()],
        }
    }
}

impl CanisterStorage {
    #[inline(always)]
    fn left_subtree_hash(&self) -> Hash {
        fork_hash(
            &self.bucket.root_hash(),
            &self.bucket_lookup_table.root_hash(),
        )
    }

    #[inline(always)]
    fn right_subtree_hash(&self) -> Hash {
        fork_hash(
            &self.readable_canisters.root_hash(),
            &self.next_canisters.root_hash(),
        )
    }
}

impl AsHashTree for CanisterStorage {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        fork_hash(&self.left_subtree_hash(), &self.right_subtree_hash())
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree {
        fork(
            fork(
                self.bucket.as_hash_tree(),
                self.bucket_lookup_table.as_hash_tree(),
            ),
            fork(
                self.readable_canisters.as_hash_tree(),
                self.next_canisters.as_hash_tree(),
            ),
        )
    }
}

#[query]
fn get_index_canisters(arg: readable::WithWitnessArg) -> readable::GetCanistersResponse<'static> {
    let storage = ic::get::<CanisterStorage>();

    readable::GetCanistersResponse {
        canisters: storage.readable_canisters.as_slice(),
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                Pruned(storage.left_subtree_hash()),
                fork(
                    storage.readable_canisters.as_hash_tree(),
                    Pruned(storage.next_canisters.root_hash()),
                ),
            ))),
        },
    }
}

#[query]
fn get_next_canisters(arg: readable::WithWitnessArg) -> readable::GetCanistersResponse<'static> {
    let storage = ic::get::<CanisterStorage>();

    readable::GetCanistersResponse {
        canisters: storage.next_canisters.as_slice(),
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                Pruned(storage.left_subtree_hash()),
                fork(
                    Pruned(storage.readable_canisters.root_hash()),
                    storage.next_canisters.as_hash_tree(),
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
                fork(
                    Pruned(storage.bucket.root_hash()),
                    storage.bucket_lookup_table.gen_witness(arg.id),
                ),
                Pruned(storage.right_subtree_hash()),
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
                    fork(
                        storage.bucket.witness_transaction(arg.id),
                        Pruned(storage.bucket_lookup_table.root_hash()),
                    ),
                    Pruned(storage.right_subtree_hash()),
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
    let page = arg
        .page
        .unwrap_or(storage.bucket.last_page_for_user(&arg.principal));

    let data = storage
        .bucket
        .get_transactions_for_user(&arg.principal, page);

    readable::GetTransactionsResponse {
        data,
        page,
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                fork(
                    storage
                        .bucket
                        .witness_transactions_for_user(&arg.principal, page),
                    Pruned(storage.bucket_lookup_table.root_hash()),
                ),
                Pruned(storage.right_subtree_hash()),
            ))),
        },
    }
}

#[query]
fn get_token_transactions(
    arg: readable::WithPageArg,
) -> readable::GetTransactionsResponse<'static> {
    let storage = ic::get::<CanisterStorage>();
    let page = arg
        .page
        .unwrap_or(storage.bucket.last_page_for_token(&arg.principal));

    let data = storage
        .bucket
        .get_transactions_for_token(&arg.principal, page);

    readable::GetTransactionsResponse {
        data,
        page,
        witness: match arg.witness {
            false => None,
            true => Some(Witness::new(fork(
                fork(
                    storage
                        .bucket
                        .witness_transactions_for_token(&arg.principal, page),
                    Pruned(storage.bucket_lookup_table.root_hash()),
                ),
                Pruned(storage.right_subtree_hash()),
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
