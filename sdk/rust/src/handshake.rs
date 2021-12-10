use std::str::FromStr;

use cap_sdk_core::{Index, Router};
use ic_kit::{
    ic,
    interfaces::{management, Method},
    Principal,
};

use crate::CapEnv;

/// Handshakes with Cap. This either grabs important metadata or
/// creates the root bucket for this contract and gives it `creation_cycles`
/// cycles.
pub fn handshake(creation_cycles: u64, router_override: Option<Principal>) {
    let router_pid = router_override
        .unwrap_or_else(|| Principal::from_str("lj532-6iaaa-aaaah-qcc7a-cai").unwrap());
    let router = Router::new(router_pid);

    let create_settings = management::CanisterSettings {
        controllers: Some(vec![router_pid]),
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
    };

    let arg = management::CreateCanisterArgument {
        settings: Some(create_settings),
    };

    let closure = async move {
        let index: Index = router.into();

        if let Ok(bucket) = index.get_token_contract_root_bucket(ic::id()).await {
            CapEnv::store(&CapEnv::create(bucket, router));
        } else {
            let (res,) = management::CreateCanister::perform_with_payment(
                Principal::management_canister(),
                (arg,),
                creation_cycles,
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
