# Root Bucket Interface

This interface extends the [*Bucket Interface*](./bucket-interface.md), and is the main
canister that each token contract can integrate with to insert their transactions to the
Cap's network.

## update insert

The insert method can be invoked by token contract that is granted permission on this root
bucket to perform insertion for a single event.