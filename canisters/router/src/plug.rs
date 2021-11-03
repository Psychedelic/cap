use crate::installer::install_code;
use crate::Data;
use ic_kit::candid::candid_method;
use ic_kit::ic;
use ic_kit::interfaces::{management, Method};
use ic_kit::macros::update;
use ic_kit::Principal;
use lazy_static::lazy_static;

lazy_static! {
    static ref PLUG_PROXY_ID: Principal = {
        Principal::from_text("qti3e-ren42-maxnk-dwpe5-h4hhi-zgnmd-fm4ak-o2vfg-64r7w-al6hm-zqe")
            .unwrap()
    };
}

#[update]
#[candid_method(update)]
async fn deploy_plug_bucket(contract_id: Principal, cycles: u64) {
    let caller = ic::caller();

    if caller != *PLUG_PROXY_ID {
        panic!("Non authorized caller.")
    }

    let data = ic::get_mut::<Data>();

    if data.root_buckets.get(&contract_id).is_some() {
        panic!(
            "Contract {} is already registered with a root bucket.",
            contract_id
        );
    }

    let arg = management::CreateCanisterArgument { settings: None };
    let (res,) = management::CreateCanister::perform_with_payment(
        Principal::management_canister(),
        (arg,),
        cycles,
    )
    .await
    .expect("Failed to create the canister.");
    let canister_id = res.canister_id;

    install_code(canister_id, contract_id, &[*PLUG_PROXY_ID]).await;
}
