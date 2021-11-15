use crate::Data;
use ic_kit::ic;
use ic_kit::macros::{post_upgrade, pre_upgrade};

#[pre_upgrade]
fn pre_upgrade() {
    let data = ic::get::<Data>();
    ic::stable_store((data,)).expect("Failed to write data to stable storage.");
}

#[post_upgrade]
fn post_upgrade() {
    if let Ok((data,)) = ic::stable_restore::<(Data,)>() {
        ic::store(data);
    }
}
