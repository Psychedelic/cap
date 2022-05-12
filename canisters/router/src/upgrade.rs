use crate::{get_user_root_buckets, Data};
use cap_common::{GetUserRootBucketsArg, RootBucketId};
use certified_vars::{Hash, Seq};
use ic_cdk::api::stable::StableReader;
use ic_kit::candid::{candid_method, encode_args, CandidType};
use ic_kit::ic;
use ic_kit::macros::{post_upgrade, pre_upgrade, update};
use ic_kit::Principal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;

#[derive(Default, CandidType, Serialize, Deserialize)]
struct RootBucketsToUpgrade(Vec<RootBucketId>);

#[derive(Serialize, Deserialize)]
pub struct DataV0 {
    pub root_buckets: BTreeMap<Vec<u8>, Vec<u8>>,
    /// Map each user to RootBucketId
    pub user_canisters: BTreeMap<Vec<u8>, CanisterListV0>,
    /// List of the index canisters.
    pub index_canisters: CanisterListV0,
}

#[derive(Default, Deserialize, Serialize)]
pub struct CanisterListV0 {
    data: Vec<Principal>,
    hash: Hash,
}

#[pre_upgrade]
fn pre_upgrade() {
    ic::stable_store((ic::get::<Data>(), ic::get::<RootBucketsToUpgrade>()))
        .expect("Failed to serialize data.");
}

#[post_upgrade]
fn post_upgrade() {
    let reader = StableReader::default();
    let data: DataV0 = match serde_cbor::from_reader(reader) {
        Ok(t) => t,
        Err(err) => {
            let limit = err.offset() - 1;
            let reader = StableReader::default().take(limit);
            serde_cbor::from_reader(reader).expect("Failed to deserialize.")
        }
    };

    let mut deserialized = Data::default();

    for (key, value) in data.root_buckets {
        let key = Principal::from_slice(&key);
        let value = Principal::from_slice(&value);
        deserialized.root_buckets.insert(key, value);
    }

    for (key, value) in data.user_canisters {
        let key = Principal::from_slice(&key);
        let value = {
            let mut r = Seq::new();
            for v in value.data {
                r.append(v);
            }
            r
        };

        deserialized.user_canisters.insert(key, value);
    }

    deserialized.index_canisters = data.index_canisters.data.into_iter().collect();

    ic::store::<Data>(deserialized);

    let root_buckets = get_user_root_buckets(GetUserRootBucketsArg {
        user: Principal::management_canister(),
        witness: false,
    })
    .contracts;

    ic::store(RootBucketsToUpgrade(root_buckets.to_vec()));
}

// #[update]
// #[candid_method(update)]
// fn trigger_upgrade() {
//     let canisters = ic::get_mut::<RootBucketsToUpgrade>();
//
//     if canisters.0.is_empty() {
//         return;
//     }
//
//     for _ in 0..16 {
//         if let Some(canister_id) = canisters.0.pop() {
//             ic_cdk::block_on(upgrade_root_bucket(canister_id));
//         } else {
//             break;
//         }
//     }
// }
//
// async fn upgrade_root_bucket(canister_id: Principal) {
//     use crate::installer::{InstallCodeArgumentBorrowed, WASM};
//     use ic_kit::interfaces::management::InstallMode;
//
//     let arg = encode_args(()).expect("Failed to serialize upgrade arg");
//     let install_config = InstallCodeArgumentBorrowed {
//         mode: InstallMode::Upgrade,
//         canister_id,
//         wasm_module: WASM,
//         arg,
//     };
//
//     // Mr. Fox
//     //            /\    /\
//     if ic::call::<_, (), _>(
//         //        \_______/
//         Principal::management_canister(),
//         "install_code",
//         (install_config,),
//     )
//     .await
//     .is_err()
//     {
//         // Retry.
//         let canisters = ic::get_mut::<RootBucketsToUpgrade>();
//         canisters.0.push(canister_id);
//     }
//
//     trigger_upgrade();
// }

#[update]
#[candid_method(update)]
async fn custom_upgrade_root_bucket(canister_id: Principal, wasm: Vec<u8>) -> bool {
    use crate::installer::InstallCodeArgumentBorrowed;
    use ic_kit::interfaces::management::InstallMode;

    let parsa =
        Principal::from_text("qti3e-ren42-maxnk-dwpe5-h4hhi-zgnmd-fm4ak-o2vfg-64r7w-al6hm-zqe")
            .unwrap();

    if ic::caller() != parsa {
        panic!("Only Parsa can call this method.")
    }

    let arg = encode_args(()).expect("Failed to serialize upgrade arg");
    let install_config = InstallCodeArgumentBorrowed {
        mode: InstallMode::Upgrade,
        canister_id,
        wasm_module: wasm.as_slice(),
        arg,
    };

    if ic::call::<_, (), _>(
        //        \_______/
        Principal::management_canister(),
        "install_code",
        (install_config,),
    )
    .await
    .is_err()
    {
        false
    } else {
        true
    }
}
