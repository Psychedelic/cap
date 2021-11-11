use crate::Data;
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_kit::ic;
use ic_kit::macros::{pre_upgrade, post_upgrade};

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();
    let writer = StableWriter::default();
    serde_cbor::to_writer(writer, &data).expect("Failed to serialize data.");
}

#[post_upgrade]
fn post_upgrade() {
    let reader = StableReader::default();
    let data = serde_cbor::from_reader(reader).expect("Failed to deserialize");
    ic::store::<Data>(data);
}