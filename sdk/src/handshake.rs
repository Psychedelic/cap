use std::cell::Cell;

use ic_kit::{
    interfaces::{management, Method},
    Principal,
};

use crate::CapEnv;

/// Handshakes with Cap. This either grabs important metadata or
/// creates the root bucket for this contract and gives it `creation_cycles`
/// cycles.
pub fn handshake(creation_cycles: u64) {
    let arg = management::CreateCanisterArgument { settings: None };

    thread_local! {
        static CYCLES: Cell<u64> = Cell::new(0);
    }

    CYCLES.with(|cycles| cycles.set(creation_cycles));

    let closure = async {
        if let Ok(bucket) = CapEnv::index()
            .get_token_contract_root_bucket(Principal::management_canister(), false)
            .await
        {
            CapEnv::store(&CapEnv::create(bucket));
        } else {
            let cycles = CYCLES.with(|cycles| cycles.get());

            let (res,) = management::CreateCanister::perform_with_payment(
                Principal::management_canister(),
                (arg,),
                cycles,
            )
            .await
            .expect("Failed to create canister");

            let canister_id = res.canister_id;

            CapEnv::router().install_code(canister_id).await.unwrap();

            let root_bucket = CapEnv::index()
                .get_token_contract_root_bucket(Principal::management_canister(), false)
                .await
                .unwrap();

            CapEnv::store(&CapEnv::create(root_bucket));
        }
    };

    CapEnv::insert_future(Box::pin(closure))
}
