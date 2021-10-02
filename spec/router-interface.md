# Router Interface

The router interface describes the interface for the entry canister of the Cap's network.
It extends the [*Indexer Interface*](./indexer-interface.md).

## update install_code

The `install_code` update method on the router canister, can be invoked by a Token Contract
to ask for code installation on an empty canister, the canister id that is passed should be
a canister with the principal id of the router as its only controller.

This method installs the WASM binary for a canister implementing the
[*Root Bucket*](./root-bucket-interface.md) on the provided canister id, and inits it
so that the caller (i.e. token contract) to this method is the only canister that has
the permission to write to that bucket.

```rust
type UninitializedRootBucketId = Principal;

#[update]
fn install_code(canister_id: UninitializedRootBucketId) -> () { /** ... */ }
```
