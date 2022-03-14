use crate::Data;
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_kit::ic;

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();
    let writer = StableWriter::default();
    serde_cbor::to_writer(writer, &data).expect("Failed to serialize data.");
}
