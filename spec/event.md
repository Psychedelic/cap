# Events in Cap

Cap has an opinionated view on what an event can be, to make it more readable by machines and also make the format
more consumable by third-parties. But it has a format that is broad enough to be used on both fungible and non-fungible
tokens.

## Operation

The operation kind for an event. The ERC-20 `TransferFrom` can use the `Transfer` operation kind with a non-null `from`
field on the event.

```rust
enum Operation {
    Transfer,
    Approve,
    Mint,
    Burn,
}
```

## Event

An event that is stored on Cap.

```rust
struct Event {
    /// The timestamp in ms.
    pub time: u64,
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The amount of tokens that was touched in this event.
    pub amount: u64,
    /// The fee captured by the token contract.
    pub fee: u64,
    /// The transaction memo.
    pub memo: u32,
    /// The `from` field, only needs to be non-null for erc-20 TransferFrom kind of events.
    pub from: Option<Principal>,
    /// The receiver end of this transaction.
    pub to: Principal,
    /// The operation that took place.
    pub operation: Operation,
}
```

## IndefiniteEvent

The *IndefiniteEvent* type is an event without the `time` field. This type is sent from the token contract to the
root bucket.

```rust
struct IndefiniteEvent {
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The amount of tokens that was touched in this event.
    pub amount: u64,
    /// The fee captured by the token contract.
    pub fee: u64,
    /// The transaction memo.
    pub memo: u32,
    /// The `from` field, only needs to be non-null for transferFrom kind of events.
    pub from: Option<Principal>,
    /// The receiver end of this transaction.
    pub to: Principal,
    /// The operation that took place.
    pub operation: Operation,
}
```

## Hashing an event

To compute the hash of an event the following algorithm is used:

```rust
type Hash = [u8; 32];

fn hash(event: Event) -> Hash {
    let mut h = match &event.operation {
        Operation::Transfer => domain_sep("transfer"),
        Operation::Approve => domain_sep("approve"),
        Operation::Mint => domain_sep("mint"),
        Operation::Burn => domain_sep("burn"),
    };

    h.update(&event.time.to_be_bytes() as &[u8]);
    h.update(&event.amount.to_be_bytes());
    h.update(&event.fee.to_be_bytes());
    h.update(&event.memo.to_be_bytes());

    // And now all of the Principal IDs
    h.update(&event.caller);
    if let Some(from) = &event.from {
        h.update(from);
    }
    h.update(&event.to);

    h.finalize().into()
}

fn domain_sep(s: &str) -> sha2::Sha256 {
    let buf: [u8; 1] = [s.len() as u8];
    let mut h = sha2::Sha256::new();
    h.update(&buf[..]);
    h.update(s.as_bytes());
    h
}
```
