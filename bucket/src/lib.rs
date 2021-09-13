use crate::transaction::Event;
use ic_certified_map::{Hash, RbTree};

mod index;
mod transaction;

pub struct Bucket {
    events: Vec<Event>,
    events_hashes: RbTree<Vec<u8>, Hash>,
}
