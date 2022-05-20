use crate::migration;
use crate::Data;
use ic_kit::ic;
use ic_kit::macros::{post_upgrade, pre_upgrade};

#[pre_upgrade]
fn pre_upgrade() {
    ic::stable_store((ic::get::<Data>(),)).expect("Failed to serialize data.");
}

#[post_upgrade]
pub fn post_upgrade() {
    let mut message = "Could not decode the data.".to_string();

    match migration::from_stable::<migration::v0::Data>() {
        Ok(v0) => {
            let data = v0.migrate().migrate();
            ic::store::<Data>(data);
            return;
        }
        Err(e) => message = format!("{} - ErrV0: {}", message, e),
    }

    match migration::from_stable::<migration::v1::Data>() {
        Ok(v1) => {
            let data = v1.migrate();
            ic::store::<Data>(data);
            return;
        }
        Err(e) => message = format!("{} - ErrV1: {}", message, e),
    }

    ic::trap(&message);
}
