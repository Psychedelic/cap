use ic_history_bucket::Bucket;
use ic_history_types::{readable, writable};
use ic_kit::macros::query;

struct CanisterStorage {
    bucket: Bucket,
    index_canisters: Vec<readable::ReadableCanisterId>,
    writable_canisters: Vec<writable::WritableCanisterId>,
}
