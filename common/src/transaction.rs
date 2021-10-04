use crate::did::EventHash;
use ic_kit::candid::{CandidType, Deserialize};
use ic_kit::Principal;
use serde::Serialize;
use sha2::Digest;
use std::collections::BTreeSet;

/// An event that took place in the transaction history of a token.
///
/// The main difference between this type and [`IndefiniteEvent`] is that
/// this event happened at a defined point in time.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    /// The timestamp in ms.
    time: u64,
    /// The caller that initiated the call on the token contract.
    caller: Principal,
    /// The amount of tokens that was touched in this event.
    amount: u64,
    /// The fee captured by the token contract.
    fee: u64,
    /// The transaction memo.
    memo: u32,
    /// The source of this transaction.
    ///
    /// Must not be [`None` if the `operation` is [`Operation::Transfer`].
    from: Option<Principal>,
    /// The receiver end of this transaction.
    to: Principal,
    /// The operation that took place.
    operation: Operation,
}

impl Event {
    /// Creates a new [`Event`]
    ///
    /// # Panics
    /// Panics if the `operation` is [`Operation::Transfer`] and `from` is [`None`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: u64,
        caller: Principal,
        amount: u64,
        fee: u64,
        memo: u32,
        from: Option<Principal>,
        to: Principal,
        operation: Operation,
    ) -> Self {
        // Since the specification states that all Operation::Transfer must have a source, we panic
        // if the caller has not provided one.
        if operation == Operation::Transfer && from.is_none() {
            panic!("A transfer operation must have a source.");
        } else {
            Self {
                time,
                caller,
                amount,
                fee,
                memo,
                from,
                to,
                operation,
            }
        }
    }

    /// Return a set containing all of the Principal IDs involved in an event.
    #[inline]
    pub fn extract_principal_ids(&self) -> BTreeSet<&Principal> {
        let mut principals = BTreeSet::new();

        principals.insert(&self.caller);
        if let Some(from) = &self.from {
            principals.insert(from);
        }
        principals.insert(&self.to);

        principals
    }

    /// Compute the hash for the given event.
    pub fn hash(&self) -> EventHash {
        let mut h = match &self.operation {
            Operation::Transfer => domain_sep("transfer"),
            Operation::Approve => domain_sep("approve"),
            Operation::Mint => domain_sep("mint"),
            Operation::Burn => domain_sep("burn"),
        };

        h.update(&self.time.to_be_bytes() as &[u8]);
        h.update(&self.amount.to_be_bytes());
        h.update(&self.fee.to_be_bytes());
        h.update(&self.memo.to_be_bytes());

        // And now all of the Principal IDs
        h.update(&self.caller);
        if let Some(from) = &self.from {
            h.update(from);
        }
        h.update(&self.to);

        h.finalize().into()
    }

    #[inline]
    pub fn time(&self) -> u64 {
        self.time
    }

    #[inline]
    pub fn caller(&self) -> Principal {
        self.caller
    }

    #[inline]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    #[inline]
    pub fn fee(&self) -> u64 {
        self.fee
    }

    #[inline]
    pub fn memo(&self) -> u32 {
        self.memo
    }

    #[inline]
    pub fn from(&self) -> Option<Principal> {
        self.from
    }

    #[inline]
    pub fn to(&self) -> Principal {
        self.to
    }

    #[inline]
    pub fn operation(&self) -> Operation {
        self.operation
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Eq, PartialEq, Copy, Debug)]
pub enum Operation {
    Transfer,
    Approve,
    Mint,
    Burn,
}

/// An event that doesn't have a defined timestamp.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IndefiniteEvent {
    /// The caller that initiated the call on the token contract.
    caller: Principal,
    /// The amount of tokens that was touched in this event.
    amount: u64,
    /// The fee captured by the token contract.
    fee: u64,
    /// The transaction memo.
    memo: u32,
    /// The source for the transaction.
    ///
    /// Must not be [`None`] if the `operation` is [`Operation::Transfer`].
    from: Option<Principal>,
    /// The receiver end of this transaction.
    to: Principal,
    /// The operation that took place.
    operation: Operation,
}

impl IndefiniteEvent {
    /// Creates a new [`IndefiniteEvent`]
    ///
    /// # Panics
    /// Panics if the `operation` is [`Operation::Transfer`] and `from` is [`None`].
    pub fn new(
        caller: Principal,
        amount: u64,
        fee: u64,
        memo: u32,
        from: Option<Principal>,
        to: Principal,
        operation: Operation,
    ) -> Self {
        // Since the specification states that all Operation::Transfer must have a source, we panic
        // if the caller has not provided one.
        if operation == Operation::Transfer && from.is_none() {
            panic!("A transfer operation must have a source.");
        } else {
            Self {
                caller,
                amount,
                fee,
                memo,
                from,
                to,
                operation,
            }
        }
    }

    // Convert an indefinite event to a definite one by adding the token and time fields.
    #[inline]
    pub fn to_event(self, time: u64) -> Event {
        Event {
            time,
            caller: self.caller,
            amount: self.amount,
            fee: self.fee,
            memo: self.memo,
            from: self.from,
            to: self.to,
            operation: self.operation,
        }
    }

    #[inline]
    pub fn caller(&self) -> Principal {
        self.caller
    }

    #[inline]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    #[inline]
    pub fn fee(&self) -> u64 {
        self.fee
    }

    #[inline]
    pub fn memo(&self) -> u32 {
        self.memo
    }

    #[inline]
    pub fn from(&self) -> Option<Principal> {
        self.from
    }

    #[inline]
    pub fn to(&self) -> Principal {
        self.to
    }

    #[inline]
    pub fn operation(&self) -> Operation {
        self.operation
    }
}

fn domain_sep(s: &str) -> sha2::Sha256 {
    let buf: [u8; 1] = [s.len() as u8];
    let mut h = sha2::Sha256::new();
    h.update(&buf[..]);
    h.update(s.as_bytes());
    h
}

// TODO(qti3e) Test
