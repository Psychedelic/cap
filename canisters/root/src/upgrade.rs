use crate::migration::{v0, v1, v2};
use crate::Data;
use crate::{migration, InProgressReadFromStable};
use ic_cdk::spawn;
use ic_kit::macros::{post_upgrade, pre_upgrade, update};
use ic_kit::{ic, Principal};
use std::collections::HashSet;

#[pre_upgrade]
fn pre_upgrade() {
    // If data doesn't exits, don't rewrite the stable store.
    if let Some(data) = ic::get_maybe::<Data>() {
        ic::stable_store((data,)).expect("Failed to serialize data.");
    }
}

// Currently all of the Cap bucket's are on the following git hashes:
// 0df4c75beaf3afe00f7d360ba6c5ef955e22a3c3
// 13d2e5dcbea5eb7c42f68bb011befb07bb543eb8
// 77144ab9463fc6dab5acc0ebd099e0efeec23cf0
//
// Except these canisters:
// 3qxje-uqaaa-aaaah-qcn4q-cai  - Sonics canister
// whq4n-xiaaa-aaaam-qaazq-cai  - WICPs canister
#[post_upgrade]
pub fn post_upgrade() {
    let from_v0 = ["3qxje-uqaaa-aaaah-qcn4q-cai", "whq4n-xiaaa-aaaam-qaazq-cai"]
        .iter()
        .map(|text| Principal::from_text(text).unwrap())
        .collect::<HashSet<_>>();

    let more_than_10k = [
        "riufi-uyaaa-aaaam-qaaiq-cai",
        "ugjiu-taaaa-aaaam-qaaua-cai",
        "mq76s-ciaaa-aaaah-qc2va-cai",
        "j5ua3-6yaaa-aaaak-aaggq-cai",
        "q675d-biaaa-aaaam-qaanq-cai",
        "mm3ed-viaaa-aaaah-qc2xa-cai",
        "he2ur-tqaaa-aaaan-qabja-cai",
        "6rjbk-7qaaa-aaaah-qczvq-cai",
        "eineg-yqaaa-aaaan-qabda-cai",
        "szgqq-gyaaa-aaaab-qaebq-cai",
        "m5na3-faaaa-aaaan-qaawa-cai",
        "bqswi-zaaaa-aaaah-abkza-cai",
        "ebop2-oyaaa-aaaan-qabcq-cai",
        "rrr4x-hqaaa-aaaam-qamga-cai",
        "mm3ed-viaaa-aaaah-qc2xa-cai",
        "myux7-2yaaa-aaaap-aah3q-cai",
    ]
    .iter()
    .map(|text| Principal::from_text(text).unwrap())
    .collect::<HashSet<_>>();

    let id = ic::id();

    if from_v0.contains(&id) {
        rescue().unwrap_or_else(|msg| ic::trap(&format!("Rescue failed: {}", msg)));
        return;
    }

    if more_than_10k.contains(&id) {
        let (data,): (v2::Data,) = ic::stable_restore().unwrap_or_else(|m| {
            ic::trap(&format!(
                "M10K: Could not deserialize data as v2::Data: {}",
                m
            ))
        });

        ic::store(InProgressReadFromStable::new(data));

        return;
    }

    let data: (Data,) = ic::stable_restore().expect("Failed to deserialize");
    ic::store(data);
}

fn rescue() -> Result<(), String> {
    let mut message = String::new();

    match migration::from_stable::<v0::Data>() {
        Ok(v0) => {
            let data = v0.migrate().migrate();
            ic::store(InProgressReadFromStable::new(data));
            return Ok(());
        }
        Err(e) => message = format!("{} - ErrV0: {}", message, e),
    }

    match migration::from_stable::<v1::Data>() {
        Ok(v1) => {
            let data = v1.migrate();
            ic::store(InProgressReadFromStable::new(data));
            return Ok(());
        }
        Err(e) => message = format!("{} - ErrV0: {}", message, e),
    }

    Err(message)
}

/// Perform the leftover tasks from the upgrade.
#[update]
pub fn upgrade_progress() {
    {
        if !ic::get_maybe::<InProgressReadFromStable>().is_some() {
            return;
        }

        let c = ic::get_mut::<InProgressReadFromStable>();
        c.progress(10000);

        if c.is_complete() {
            let data = c.get_data().unwrap();
            ic::store(data);
            ic::delete::<InProgressReadFromStable>();
            return;
        }
    }

    for _ in 0..4 {
        spawn(async {
            match ic::call::<(), (), &str>(ic::id(), "upgrade_progress", ()).await {
                Ok(_) => {}
                Err(_) => {}
            }
        });
    }
}
