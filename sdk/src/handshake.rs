use std::{cell::Cell, str::FromStr};

use cap_sdk_core::{Index, Router};
use ic_kit::{Principal, ic, interfaces::{management, Method}};

use crate::CapEnv;

/// Handshakes with Cap. This either grabs important metadata or
/// creates the root bucket for this contract and gives it `creation_cycles`
/// cycles.
pub fn handshake(creation_cycles: u64, router_override: Option<Principal>) {
    let arg = management::CreateCanisterArgument { settings: None };

    let router = {
        if let Some(router_override) = router_override {
            Router::new(router_override)
        } else {
            Router::new(Principal::from_str("lj532-6iaaa-aaaah-qcc7a-cai").unwrap())
        }
    };

    // Used to bypass the fact that `creation_cycles` has to be `'static` which it isn't.
    thread_local! {
        static CYCLES: Cell<u64> = Cell::new(0);
        static ROUTER: Cell<Option<Router>> = Cell::new(None);
    }

    CYCLES.with(|cycles| cycles.set(creation_cycles));
    ROUTER.with(|router_cell| router_cell.set(Some(router)));

    let closure = async {
        let router = ROUTER.with(|router_cell| router_cell.get().take().unwrap());

        let index: Index = router.into();

        if let Ok(bucket) = index
            .get_token_contract_root_bucket(ic::id())
            .await
        {
            CapEnv::store(&CapEnv::create(bucket, router));
        } else {
            let cycles = CYCLES.with(|cycles| cycles.get());

            let (res,) = management::CreateCanister::perform_with_payment(
                ic::id(),
                (arg,),
                cycles,
            )
            .await
            .expect("Failed to create canister");

            let canister_id = res.canister_id;

            router.install_code(canister_id).await.unwrap();

            let root_bucket = index
                .get_token_contract_root_bucket(ic::id())
                .await
                .unwrap();

            CapEnv::store(&CapEnv::create(root_bucket, router));
        }
    };

    CapEnv::insert_future(Box::pin(closure))
}
