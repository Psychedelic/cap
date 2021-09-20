use crate::readable::EventHash;
use ic_kit::candid::{CandidType, Deserialize};
use ic_kit::Principal;
use serde::Serialize;
use sha2::Digest;
use std::collections::BTreeSet;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    /// The canister that inserted this event to the history.
    pub contract: Principal,
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
    /// The `from` field, only needs to be non-null for transferFrom kind of events.
    pub from: Option<Principal>,
    /// The receiver end of this transaction.
    pub to: Principal,
    /// The operation that took place.
    pub operation: Operation,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum Operation {
    Transfer,
    Approve,
    Mint,
    Burn,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IndefiniteEvent {
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

impl Event {
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
        h.update(&self.contract);
        h.update(&self.caller);
        if let Some(from) = &self.from {
            h.update(from);
        }
        h.update(&self.to);

        h.finalize().into()
    }
}

impl IndefiniteEvent {
    /// Convert an indefinite event to a definite one by adding the token and time fields.
    #[inline]
    pub fn to_event(self, contract: Principal, time: u64) -> Event {
        Event {
            contract,
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
}

fn domain_sep(s: &str) -> sha2::Sha256 {
    let buf: [u8; 1] = [s.len() as u8];
    let mut h = sha2::Sha256::new();
    h.update(&buf[..]);
    h.update(s.as_bytes());
    h
}

// TODO(qti3e) Test
