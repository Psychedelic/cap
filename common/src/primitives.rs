use ic_certified_map::Hash;
use ic_kit::Principal;

/// The numeric type used to represent a transaction id.
pub type TransactionId = u64;

/// The principal id of a index canister.
pub type IndexCanisterId = Principal;

/// The principal id of a root bucket canister.
pub type RootBucketId = Principal;

/// The principal id of a bucket canister.
pub type BucketId = Principal;

/// The principal id of a token contract this is integrating Cap.
pub type TokenContractId = Principal;

/// Hash of an even which is obtained by `Event::hash`
pub type EventHash = Hash;
