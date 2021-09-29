# Internet Computer History OIS Spec

This document goes into the interface description of the Internet Computer History OIS
without implementation. To keep the name short we refer to this service as ICHS in this
document.

The ICHS is a service which provides scalable transaction history that can be used by any
number of tokens or NFTs to store their event log and issue a transaction id to the end
user. It also provides a unified view to the history for every token contract that
integrates this service for client-facing wallets and network scan UIs.

The primary goal of this project is filling the gap for a native unified ETH-like history
on the Internet Computer.

In this document we go thought the overall architecture of such a service and provide the
schema for the canisters that the system has.

The goal of defining this interfaces is to have implementation agnostic layer of
interaction with this open internet service and future proofing the OIS.

## Certified Trees

Each canister should have one merkle-tree whose root hash is stored as the certified data
of the entire canister.

This tree is defined using the following structure:

```
HashTree
  = Empty
  | Fork HashTree HashTree
  | Labeled Label HashTree
  | Leaf blob
  | Pruned Hash
Label = Blob
Hash = Blob
Signature = Blob
```

The schema above describes a tree which has binary blobs as key and values of the tree.
Any number inserted as either label or leaves on tree should be stored in Big Endian.

We define the following operations on the `HashTree`:

1. `reconstruct` should provide the root hash of the tree.
```
reconstruct(Empty)       = H(domain_sep("ic-hashtree-empty"))
reconstruct(Fork t1 t2)  = H(domain_sep("ic-hashtree-fork") · reconstruct(t1) · reconstruct(t2))
reconstruct(Labeled l t) = H(domain_sep("ic-hashtree-labeled") · l · reconstruct(t))
reconstruct(Leaf v)      = H(domain_sep("ic-hashtree-leaf") · v)
reconstruct(Pruned h)    = h

domain_sep(s) = byte(|s|) · s
```

2. `flatten` should eliminate the pruned nodes and return the most inner tree, of course
the new tree will have a different root hash, so the tree obtained should already be
certified.
```
flatten(Fork Pruned t) = flatten(t)
flatten(Fork t Pruned) = flatten(t)
flatten(t) = t
```

The alias `tree<K, V>` in this doc refers to a tree with labels of type `K` and leaves of
type `V`.

Type alias `leaf<T>` refers to a tree node that has the given data type.

The `flatten_fork<T1, T2>` refers to a tree that once flattened, it'll have two sub-trees
with the given types, the `T1` is the left/first subtree and `T2` is the right/second
tree.

For example `flatten_fork<tree<u32, TransactionHash>, leaf<u64>>` means that the most
inner subtree of the root tree should be a fork that has two nodes, one of which is
another tree that maps `u32` values to a `TransactionHash` and the second node is a
`u64` constant number.

## Transactions

The transaction type determines the shape of each transaction that can be inserted or
queried from the ICHS service.

A transaction is described by the following candid interface:

```
type Event = variant {
    // The original caller to the `insert` method.
    contract : principal;
    // The time the transaction was inserted to ICHS in ms.
    time     : u64;
    // Should be the original caller who invoked the call on the token canister.
    caller   : principal;
    // The amount touched in the event.
    amount   : u64;
    // The fee that was captured by the token contract.
    fee      : u64;
    // A memo for this transaction.
    memo     : u32;
    // The `from` field, only needs to be non-null for transferFrom kind of events.
    from     : opt principal,
    // The receiver end of this transaction.
    to       : principal,
    // The operation that took place.
    operation: Operation,
};

type IndefiniteEvent = variant {
    caller    : principal;
    amount    : u64;
    fee       : u64;
    memo      : u32;
    from      : opt principal,
    to        : principal,
    operation : Operation,
};

type Operation = variant {
    Transfer,
    Approve,
    Mint,
    Burn,
};
```

Now we describe how you can obtain a hash from a `Event`, the most important rule is that
every field in the `Event` should be part of the process of generating the hash.

```
domain_sep_for_operation(Transfer) = domain_sep("transfer")
domain_sep_for_operation(Approve) = domain_sep("approve")
domain_sep_for_operation(Mint) = domain_sep("mint")
domain_sep_for_operation(Burn) = domain_sep("burn")

hash_event(Event contract time caller amount fee memo NIL to operation) =
    H(domain_sep_for_operation(operation)
    . byte(time) . byte(amount) . byte(fee) . byte(memo)
    . contract . caller . to)

hash_event(Event contract time caller amount fee memo from to operation) =
    H(domain_sep_for_operation(operation)
    . byte(time) . byte(amount) . byte(fee) . byte(memo)
    . contract . caller . from . to)

```

## Index canister

```candid
type TokenContractId = principal
type RootBucketId = principal;
type RouterId = principal;
type UserId = principal;

type Witness = record {
    certificate: blob;
    // CBOR serialized HashTree
    tree: blob;
};

type GetTokenContractRootBucketArg = record {
    canister: TokenContractId;
    witness: bool;
};

type GetTokenContractRootBucketResponse = record {
    canister: opt RootBucketId;
    // Witness type: tree<TokenContractId, PrincipalHash>
    witness: opt Witness
}

type GetUserRootBucketsArg = record {
    user: UserId;
    witness: bool;
};

type GetUserRootBucketsResponse = record {
    contracts: vec RootBucketId;
    // Witness type: tree<UserId, CanistersListHash>
    witness: opt Witness;
};

type WithWitnessArg = record {
    witness: bool;
};

type GetCanistersResponse = record {
    canisters: vec RouterId;
    // Witness type: leaf(CanistersListHash)
    // CanistersListHash is computed like events page.
    witness: opt Witness;
};

service : {
    // Return the root bucket canister associated with the given token contract.
    get_token_contract_root_bucket : (GetTokenContractRootBucketArg) -> (GetTokenContractRootBucketResponse) query;

    // Return the root bucket of all the token contracts a user has transactions on.
    get_user_root_buckets : (GetUserRootBucketsArg) -> (GetUserRootBucketsResponse) query;

    // Return the list of canisters that can be used for quering the indexes. THes
    get_router_canisters : (WithWitnessArg) -> (GetRouterCanistersResponse) query;
};

```

## Main Router

The main router extends the `Indexer` canister, and has the following additional methods:
It is the entry point of the ICHS service, it is an aggregation layer over all the history
buckets that exists on the network. It should facilitate creating new buckets for the token
contracts, and also provide global indexes for the users.

```candid
service : {
    // Called by a token contract: Install the bucket code on the given canister and setup
    // the caller as the writer on the bucket.
    // The given principal ID should be of a newly created bucket that has ICHS as the only
    // controller.
    // This method simply panics if the required criterias are not met.
    install_bucket_code : (RootBucketId) -> ();
};
```

## Bucket Canister

```
type ReadableCanisterId = principal;
type EventHash = blob;
type TransactionId = nat64;

type WithIdArg = record {
    id: TransactionId;
    witness: bool;
};

type GetTransactionResponse = variant {
    // Witness type: tree<TransactionId, ReadableCanisterId>
    Delegate(principal, opt Witness),
    // Witness type: flatten_fork<tree<nat32, EventHash>, leaf<TransactionId>>
    Found(Event, opt Witness)
};

// [nat8; 34] = byte(principal) . byte(nat32)
// 30 bytes for principal, the first byte is the len.
// 4 bytes for page number.
type PageKey = blob;

// Hash a page of events. See the section below for Page Hash.
type PageHash = blob;

type GetTransactionsArg = record {
    page: opt nat32;
    witness: bool;
};

type WithPageArg = record {
    principal: principal;
    page: opt nat32;
    witness: bool;
};

type GetTransactionsResponse = struct {
    data: vec Event;
    page : nat32;
    // Witness type: tree<PageKey, PageHash>
    witness: opt Witness;
};

type WithWitnessArg = record {
    witness: bool;
};

type GetRouterCanistersResponse = record {
    canisters: vec ReadableCanisterId;
    // Witness type: leaf(CanistersListHash)
    // CanistersListHash is computed like events page.
    witness: opt Witness;
}

type GetBucketResponse = record {
    canister: ReadableCanisterId;
    // Witness type: tree<TransactionId, ReadableCanisterId>
    witness: opt Witness;
};

service : {
    // Return the list of canisters to obtain more pages of data.
    get_next_canisters : (WithWitnessArg) -> (GetCanistersResponse) query;

    // Return the given transaction.
    get_transaction : (WithIdArg) -> (GetTransactionResponse) query;

    // Return all of the transactions for this contract.
    get_transactions : (GetTransactionsArg) -> (GetTransactionsResponse) query;

    // Return all of the transactions associated with the given user.
    get_user_transactions : (WithPageArg) -> (GetTransactionsResponse) query;
};
```

## Root Bucket

The root bucket extends the `Bucket`, but has some additional methods.

```candid
service root_bucket : {
    // Return a bucket that can be used to query for the given transaction id.
    get_bucket_for : (WithIdArg) -> (GetBucketResponse) query;

    // Insert the given transaction to the ICHS and issue a transaction id.
    insert : (IndefiniteEvent) -> (TransactionId);

    // The time on the canister. The time can be used to check if this bucket is
    // on the same subnet as the caller.
    time : () -> (nat64) query;
};

```

### Page Hash

```
hash_page(vec) = [0; 32]
hash_page(vec ..events event) = H(hash_page(events) . hash_event(event))
```
