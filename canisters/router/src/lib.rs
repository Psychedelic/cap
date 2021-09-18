use ic_history_common::{readable, writable, Bucket};
use ic_kit::macros::query;

struct CanisterStorage {
    bucket: Bucket,
    index_canisters: Vec<readable::ReadableCanisterId>,
    writable_canisters: Vec<writable::WritableCanisterId>,
}
