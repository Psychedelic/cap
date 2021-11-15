use cap_sdk::{DetailValue, Event, IndefiniteEventBuilder, IntoEvent};
use ic_kit::candid::CandidType;
use ic_kit::candid::{candid_method, export_service};
use ic_kit::macros::{query, update};
use ic_kit::{ic, Principal};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

mod upgrade;

/// The datastore used to hold the canister state.
#[derive(Serialize, Deserialize, CandidType)]
struct Data {
    /// Next token id that should be used.
    next_id: u64,
    /// Map each token id to the owner of it.
    nft_owners: BTreeMap<u64, Principal>,
}

/// The default implementation for the data store used to initialize
/// the data.
impl Default for Data {
    fn default() -> Self {
        Self {
            next_id: 0,
            nft_owners: BTreeMap::new(),
        }
    }
}

#[query(name = "get_nft_owner")]
#[candid_method(query)]
pub async fn get_nft_owner(token_id: u64) -> Principal {
    let data = ic::get::<Data>();
    let owner = data
        .nft_owners
        .get(&token_id)
        .expect("Error finding owner.");
    *owner
}

#[update(name = "setup_cap")]
#[candid_method(update)]
pub async fn setup_cap() {
    let cycles_to_give = 1_000_000_000_000;

    cap_sdk::handshake(
        cycles_to_give,
        Some(Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()),
    );

    // Uncomment this for main net lunch.
    // cap_sdk::handshake(cycles_togive, None);
}

/// The data structure used to store the "Mint" history event.
pub struct MintDetails {
    owner: Principal,
    token_id: u64,
    cycles: u64,
}

impl IntoEvent for MintDetails {
    fn details(&self) -> Vec<(String, DetailValue)> {
        vec![
            ("owner".into(), self.owner.into()),
            ("token_id".into(), self.token_id.into()),
            ("cycles".into(), self.cycles.into()),
        ]
    }
}

/// The structure used to encode "Transfer" events.
pub struct TransferDetails {
    to: Principal,
    token_id: u64,
}

impl IntoEvent for TransferDetails {
    fn details(&self) -> Vec<(String, DetailValue)> {
        vec![
            ("to".into(), self.to.into()),
            ("token_id".into(), self.token_id.into()),
        ]
    }
}

#[update(name = "mint")]
#[candid_method(update)]
pub async fn mint(owner: Principal) -> u64 {
    let available = ic::msg_cycles_available();
    let fee = 2_000_000_000_000;

    ic::print(format!("Available cycles: {}", available));

    if available < fee {
        panic!(
            "Can not mint: {} provided cycles is less than the required fee of {}",
            available, fee
        );
    }

    let data = ic::get_mut::<Data>();
    let token_id = data.next_id;
    ic::msg_cycles_accept(fee);

    let transaction_details = MintDetails {
        owner,
        token_id,
        cycles: available,
    };

    data.nft_owners.insert(transaction_details.token_id, owner);
    data.next_id += 1;

    let event = IndefiniteEventBuilder::new()
        .caller(ic::caller())
        .operation(String::from("mint"))
        .details(transaction_details)
        .build()
        .unwrap();

    cap_sdk::insert(event).await.unwrap();

    token_id
}

#[update(name = "transfer")]
#[candid_method(update)]
pub async fn transfer(new_owner: Principal, token_id: u64) {
    let available = ic::msg_cycles_available();
    let fee = 1_000_000_000;

    if available < fee {
        panic!(
            "Can not transfer: {} provided cycles is less than the required fee of {}",
            available, fee
        );
    }

    ic::msg_cycles_accept(fee);
    let data = ic::get_mut::<Data>();

    let existing_owner = data
        .nft_owners
        .get(&token_id)
        .expect("Error finding owner.");

    let caller = ic::caller();

    if caller != *existing_owner {
        panic!("Not owner.");
    }

    data.nft_owners.insert(token_id, new_owner);

    let transaction_details = TransferDetails {
        to: new_owner,
        token_id,
    };

    let event = IndefiniteEventBuilder::new()
        .caller(ic::caller())
        .operation(String::from("transfer"))
        .details(transaction_details)
        .build()
        .unwrap();

    cap_sdk::insert(event).await.unwrap();
}

#[candid_method(update)]
#[update(name = "get_transaction_by_id")]
pub async fn get_transaction_by_id(id: u64) -> Event {
    cap_sdk::get_transaction(id)
        .await
        .expect("Error retrieving transaction")
}

// needed to export candid on save
#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let dir = dir.parent().unwrap().parent().unwrap().join("candid");
        write(dir.join("sdk_example.did"), export_candid()).expect("Write failed.");
    }
}
