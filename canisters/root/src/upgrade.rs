use crate::Data;
use ic_cdk::api::stable::StableReader;
use ic_kit::ic;
use ic_kit::macros::{post_upgrade, pre_upgrade};
use std::io::Read;

#[pre_upgrade]
fn pre_upgrade() {
    ic::stable_store((ic::get::<Data>(),)).expect("Failed to serialize data.");
}

#[post_upgrade]
pub fn post_upgrade() {
    let reader = StableReader::default();

    let data: Data = match serde_cbor::from_reader(reader) {
        Ok(t) => t,
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).expect("Failed to deserialize.")
        }
    };

    ic::store(data);
}
