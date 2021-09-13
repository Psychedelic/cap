use ic_certified_map::Hash;
use ic_kit::candid::{CandidType, Deserialize};
use ic_kit::Principal;
use sha2::Digest;
use std::collections::BTreeSet;

#[derive(CandidType, Deserialize, Clone)]
pub struct Event {
    /// The canister that inserted this event to the history.
    pub token: Principal,
    /// The timestamp in ms.
    pub time: u64,
    /// The caller that initiated the call on the token.
    pub caller: Principal,
    /// The amount of tokens that was touched in this event.
    pub amount: u64,
    /// The fee captured by the token contract.
    pub fee: u64,
    /// The transaction memo.
    pub memo: u32,
    /// The transaction detail
    pub kind: EventKind,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum EventKind {
    Transfer {
        from: Principal,
        to: Principal,
    },
    Mint {
        to: Principal,
    },
    Burn {
        from: Principal,
        to: Option<Principal>,
    },
    Custom {
        name: String,
        spenders: Vec<Principal>,
        receivers: Vec<Principal>,
    },
}

impl Event {
    /// Return a set containing all of the Principal IDs involved in an event.
    #[inline]
    pub fn extract_principal_ids(&self) -> BTreeSet<&Principal> {
        let mut principals = BTreeSet::new();

        principals.insert(&self.caller);
        match &self.kind {
            EventKind::Transfer { from, to } => {
                principals.insert(from);
                principals.insert(to);
            }
            EventKind::Mint { to } => {
                principals.insert(to);
            }
            EventKind::Burn { from, to } => {
                principals.insert(from);
                if let Some(to) = to {
                    principals.insert(to);
                }
            }
            EventKind::Custom {
                spenders,
                receivers,
                ..
            } => {
                for id in spenders {
                    principals.insert(id);
                }

                for id in receivers {
                    principals.insert(id);
                }
            }
        }

        principals
    }

    /// Compute the hash for the given event.
    pub fn hash(&self) -> Hash {
        let mut h = match &self.kind {
            EventKind::Transfer { .. } => domain_sep("transfer"),
            EventKind::Mint { .. } => domain_sep("mint"),
            EventKind::Burn { .. } => domain_sep("burn"),
            EventKind::Custom { .. } => domain_sep("custom"),
        };

        h.update(&self.time.to_be_bytes() as &[u8]);
        h.update(&self.amount.to_be_bytes());
        h.update(&self.fee.to_be_bytes());
        h.update(&self.memo.to_be_bytes());

        // And now all of the Principal IDs
        h.update(&self.token);
        h.update(&self.caller);

        match &self.kind {
            EventKind::Transfer { from, to } => {
                h.update(from);
                h.update(to);
            }
            EventKind::Mint { to } => {
                h.update(to);
            }
            EventKind::Burn { from, to } => {
                h.update(from);
                if let Some(to) = to {
                    h.update(to);
                }
            }
            EventKind::Custom {
                spenders,
                receivers,
                ..
            } => {
                for id in spenders {
                    h.update(id);
                }
                for id in receivers {
                    h.update(id);
                }
            }
        }

        h.finalize().into()
    }
}

fn domain_sep(s: &str) -> sha2::Sha256 {
    let buf: [u8; 1] = [s.len() as u8];
    let mut h = sha2::Sha256::new();
    h.update(&buf[..]);
    h.update(s.as_bytes());
    h
}
