# Root Bucket Interface

This interface extends the [*Bucket Interface*](./bucket-interface.md), and is the main
canister that each token contract can integrate with to insert their transactions to the
Cap's network.

## update insert

The insert method can be invoked by token contract that is granted permission on this root
bucket to perform insertion for a single event.

see:
[`IndefiniteEvent`](./event.md#IndefiniteEvent)

```rust
type TransactionId = u64;

#[update]
fn insert(event: IndefiniteEvent) -> TransactionId { /** ... */ }
```

## query get_bucket_for

Returns the bucket that should contain a certain transaction id.

```rust
type TransactionId = u64;
type BucketId = Principal;

struct WithIdArg {
    /// TransactionId that we're searching for.
    pub id: TransactionId,
    /// Determines if the response should contain a witness.
    pub witness: bool,
}

struct GetBucketResponse {
    pub canister: BucketId,
    /// Witness tree structure:  
    /// label: [u8; 8]  TransactionId serialized as big-endian bytes.  
    /// leaf : [u8; 32] Sha256(BucketId)  
    ///  
    /// The witness is generally a range witness which means it will contain two consecutive nodes
    /// where `Key(Node1) < Key(Node2)` and the hash of the returned canister id will be equal to
    /// `Value(Node1)`.
    /// 
    /// Example: Imagine we're trying to query which bucket contains transaction 1130. Suppose each
    /// bucket was to store 500 transactions for the simplicity of the example. Then we'd have the
    /// following list of buckets along their starting offset.
    /// 
    /// ```
    /// [(0, Root Bucket), (500, B1), (1000, B2), (1500, B3)]
    /// ```
    /// 
    /// To verify which canister contains transaction 1130, we would have to return a sub-tree of
    /// the full merkle tree, that 1130 in the middle. So if we returned 3-rd item and the 4-th
    /// in the list, then we would have a complete verification.
    /// 
    /// So by this tree we prove that there is no other canister in between the two neighbours
    /// that were returned.
    /// 
    /// In some case where `id` equals to an starting offset, then the tree will have only one
    /// node with `Key(Node) = BigEndian(id)`.
    pub witness: Option<Witness>,
}

#[query]
fn get_bucket_for(arg: WithIdArg) -> GetBucketResponse { /** */ }
```

## query time

Return the canister time in nanoseconds.

```rust
#[query]
fn time() -> u64 { ic::time() }
```