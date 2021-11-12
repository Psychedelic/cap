use crate::{get_user_root_buckets, Data};
use ic_cdk::api::stable::{StableReader, StableWriter};
use ic_history_common::{GetUserRootBucketsArg, RootBucketId};
use ic_kit::candid::{candid_method, encode_args};
use ic_kit::ic;
use ic_kit::macros::{post_upgrade, pre_upgrade, update};
use ic_kit::Principal;

#[derive(Default)]
struct RootBucketsToUpgrade(Vec<RootBucketId>);

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

    let root_buckets = get_user_root_buckets(GetUserRootBucketsArg {
        user: Principal::management_canister(),
        witness: false,
    })
    .contracts;

    ic::store(RootBucketsToUpgrade(root_buckets.to_vec()));
}

#[update]
#[candid_method(update)]
fn trigger_upgrade() {
    let canisters = ic::get_mut::<RootBucketsToUpgrade>();

    if canisters.0.is_empty() {
        return;
    }

    for _ in 0..16 {
        if let Some(canister_id) = canisters.0.pop() {
            ic_cdk::block_on(upgrade_root_bucket(canister_id));
        } else {
            break;
        }
    }
}

async fn upgrade_root_bucket(canister_id: Principal) {
    use crate::installer::{InstallCodeArgumentBorrowed, WASM};
    use ic_kit::interfaces::management::InstallMode;

    let arg = encode_args(()).expect("Failed to serialize upgrade arg");
    let install_config = InstallCodeArgumentBorrowed {
        mode: InstallMode::Upgrade,
        canister_id,
        wasm_module: WASM,
        arg,
    };

    // Mr. Fox
    //            /\    /\
    if ic::call::<_, (), _>(
        //        \_______/
        Principal::management_canister(),
        "install_code",
        (install_config,),
    )
    .await
    .is_err()
    {
        // Retry.
        let canisters = ic::get_mut::<RootBucketsToUpgrade>();
        canisters.0.push(canister_id);
    }

    trigger_upgrade();
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_kit::MockContext;

    const fn p(id: u8) -> Principal {
        Principal::from_slice(&[id, 0x00])
    }

    #[test]
    fn test_p() {
        let principal = p(67);
        let serialized = serde_cbor::to_vec(&principal).expect("Failed to serialize.");
        let actual: Principal =
            serde_cbor::from_slice(&serialized).expect("Failed to deserialize.");
        assert_eq!(principal, actual);
    }

    #[test]
    fn test() {
        let contract_1 = p(0);
        let rb_1 = p(1);
        let contract_2 = p(2);
        let rb_2 = p(3);
        let alice = p(4);
        let bob = p(5);

        MockContext::new().with_id(p(17)).inject();

        let mut data = Data::default();
        data.root_buckets.insert(contract_1, rb_1);
        data.root_buckets.insert(contract_2, rb_2);
        data.user_canisters.insert(alice, rb_1);
        data.user_canisters.insert(alice, rb_2);
        data.user_canisters.insert(bob, rb_2);

        let serialized = serde_cbor::to_vec(&data).expect("Failed to serialize.");
        let actual: Data = serde_cbor::from_slice(&serialized).expect("Failed to deserialize.");
    }
}
