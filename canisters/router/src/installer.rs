use crate::Data;
use ic_kit::candid::candid_method;
use ic_kit::candid::encode_args;
use ic_kit::candid::CandidType;
use ic_kit::interfaces::management::InstallMode;
use ic_kit::interfaces::{management, Method};
use ic_kit::{ic, Principal};
use serde::Deserialize;

// It's ok.
use ic_history_common::*;
use ic_kit::macros::*;

#[cfg(debug_cfg)]
pub const WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/debug/ic_history_root-deb-opt.wasm");

#[cfg(not(debug_cfg))]
pub const WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/ic_history_root-rel-opt.wasm");

#[derive(CandidType, Deserialize)]
pub struct InstallCodeArgumentBorrowed<'a> {
    pub mode: InstallMode,
    pub canister_id: Principal,
    #[serde(with = "serde_bytes")]
    pub wasm_module: &'a [u8],
    pub arg: Vec<u8>,
}

#[update]
#[candid_method(update)]
async fn install_bucket_code(canister_id: RootBucketId) {
    let contract_id = ic::caller();
    install_code(canister_id, contract_id, &[]).await;
}

pub async fn install_code(canister_id: Principal, contract_id: Principal, writers: &[Principal]) {
    use management::{CanisterStatus, WithCanisterId};

    let data = ic::get_mut::<Data>();

    if data.root_buckets.get(&contract_id).is_some() {
        panic!(
            "Contract {} is already registered with a root bucket.",
            contract_id
        );
    }

    let (response,) = CanisterStatus::perform(
        Principal::management_canister(),
        (WithCanisterId { canister_id },),
    )
    .await
    .expect("Failed to retrieve canister status");

    if response.settings.controllers.len() > 1 {
        panic!("Expected one controller on canister {}", canister_id);
    }

    if response.module_hash.is_some() {
        panic!(
            "Expected an empty canister. Canister {} already has an installed wasm on it.",
            canister_id
        );
    }

    let arg =
        encode_args((contract_id, writers)).expect("Failed to serialize the install argument.");

    let install_config = InstallCodeArgumentBorrowed {
        mode: InstallMode::Install,
        canister_id,
        wasm_module: WASM,
        arg,
    };

    let _: () = ic::call(
        Principal::management_canister(),
        "install_code",
        (install_config,),
    )
    .await
    .expect("Install code failed.");

    data.root_buckets.insert(contract_id, canister_id);

    data.user_canisters
        .insert(Principal::management_canister(), canister_id);
}
