use crate::did::EventHash;
use ic_kit::candid::{CandidType, Deserialize};
use ic_kit::Principal;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    /// The timestamp in ms.
    pub time: u64,
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The status of the event, can be either `running`, `completed` or `failed`.
    pub status: EventStatus,
    /// The operation that took place.
    pub operation: String,
    /// Details of the transaction.
    pub details: Vec<(String, DetailValue)>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum EventStatus {
    Running,
    Completed,
    Failed,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IndefiniteEvent {
    /// The caller that initiated the call on the token contract.
    pub caller: Principal,
    /// The status of the event, can be either `running`, `completed` or `failed`.
    pub status: EventStatus,
    /// The operation that took place.
    pub operation: String,
    /// Details of the transaction.
    pub details: Vec<(String, DetailValue)>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum DetailValue {
    U64(u64),
    I64(i64),
    Float(f64),
    Text(String),
    Principal(Principal),
    #[serde(with = "serde_bytes")]
    Slice(Vec<u8>),
    Vec(Box<Vec<DetailValue>>),
}

impl Event {
    /// Return a set containing all of the Principal IDs involved in an event.
    #[inline]
    pub fn extract_principal_ids(&self) -> BTreeSet<&Principal> {
        let mut principals = BTreeSet::new();

        principals.insert(&self.caller);

        // TODO(qti3e) Support nested extractions.
        for (_, value) in &self.details {
            match value {
                DetailValue::Principal(p) => {
                    principals.insert(p);
                }
                _ => {}
            }
        }

        principals
    }

    /// Compute the hash for the given event.
    pub fn hash(&self) -> EventHash {
        let mut h = domain_sep(&self.operation);

        h.update(&self.time.to_be_bytes() as &[u8]);
        let caller = self.caller.as_slice();
        h.update(&caller.len().to_be_bytes() as &[u8]);
        h.update(caller);

        fn hash_value(h: &mut Sha256, value: &DetailValue) {
            match value {
                DetailValue::U64(val) => {
                    let bytes = val.to_be_bytes();
                    h.update(&[0]);
                    h.update(&bytes.len().to_be_bytes() as &[u8]);
                    h.update(bytes);
                }
                DetailValue::I64(val) => {
                    let bytes = val.to_be_bytes();
                    h.update(&[1]);
                    h.update(&bytes.len().to_be_bytes() as &[u8]);
                    h.update(bytes);
                }
                DetailValue::Float(val) => {
                    let bytes = val.to_be_bytes();
                    h.update(&[2]);
                    h.update(&bytes.len().to_be_bytes() as &[u8]);
                    h.update(bytes);
                }
                DetailValue::Text(val) => {
                    let bytes = val.as_str().as_bytes();
                    h.update(&[3]);
                    h.update(&bytes.len().to_be_bytes() as &[u8]);
                    h.update(bytes);
                }
                DetailValue::Principal(val) => {
                    let bytes = val.as_slice();
                    h.update(&[4]);
                    h.update(&bytes.len().to_be_bytes() as &[u8]);
                    h.update(bytes);
                }
                DetailValue::Slice(val) => {
                    let bytes = val.as_slice();
                    h.update(&[5]);
                    h.update(&bytes.len().to_be_bytes() as &[u8]);
                    h.update(bytes);
                }
                DetailValue::Vec(val) => {
                    h.update(&[6]);
                    h.update(&val.len().to_be_bytes() as &[u8]);
                    for item in val.iter() {
                        hash_value(h, item);
                    }
                }
            }
        }

        for (key, value) in &self.details {
            h.update(&key.len().to_be_bytes() as &[u8]);
            h.update(key.as_str().as_bytes());
            hash_value(&mut h, value);
        }

        h.finalize().into()
    }
}

impl IndefiniteEvent {
    /// Convert an indefinite event to a definite one by adding the token and time fields.
    #[inline]
    pub fn to_event(self, time: u64) -> Event {
        Event {
            time,
            caller: self.caller,
            status: self.status,
            operation: self.operation,
            details: self.details,
        }
    }
}

impl From<u64> for DetailValue {
    fn from(num: u64) -> Self {
        Self::U64(num)
    }
}

impl From<i64> for DetailValue {
    fn from(num: i64) -> Self {
        Self::I64(num)
    }
}

impl From<f64> for DetailValue {
    fn from(float: f64) -> Self {
        Self::Float(float)
    }
}

impl From<String> for DetailValue {
    fn from(string: String) -> Self {
        Self::Text(string)
    }
}

impl From<Principal> for DetailValue {
    fn from(principal: Principal) -> Self {
        Self::Principal(principal)
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
