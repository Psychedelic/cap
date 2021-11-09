use cap_sdk::{
    get_transaction, handshake, insert, DetailValue, Event, IndefiniteEventBuilder, IntoEvent,
};
use ic_certified_map::{fork, fork_hash, AsHashTree, HashTree};
use ic_kit::candid::{candid_method, export_service};
use ic_kit::interfaces::{management, Method};
use ic_kit::macros::*;
use ic_kit::{
    candid::{CandidType, Int, Nat},
    get_context, ic, Context, Principal,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::str::FromStr;

// is this needed?
//mod upgrade;

// required by crate::Data;
#[derive(Serialize)]
struct Data {
    next_id: u64,
    cap_root: Principal,
    owner: Principal,
    nft_owners: BTreeMap<u64, Principal>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            next_id: 0,
            cap_root: Principal::management_canister(),
            owner: Principal::management_canister(),
            nft_owners: BTreeMap::new(),
        }
    }
}

#[init]
fn init() {
    let data = ic::get_mut::<Data>();
    data.owner = ic::caller();
}

#[query]
pub async fn get_owner() -> Principal {
    let data = ic::get::<Data>();
    data.owner
}

#[update(name = "setup_cap")]
#[candid_method(update)]
pub async fn setup_cap() {
    let cycles_to_give = 100000000000;

    handshake(
        cycles_to_give,
        Some(Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()),
    );
    //handshake(cycles_togive, None);
}

pub struct MintDetails {
    owner: Principal,
    token_id: u64,
    cycles: u64,
}

impl IntoEvent for MintDetails {
    fn details(&self) -> Vec<(String, DetailValue)> {
        let mut vec = Vec::new();
        // TODO add data to the vec
        vec
    }
}

pub struct TransferDetails {
    to: Principal,
    token_id: u64,
}

impl IntoEvent for TransferDetails {
    fn details(&self) -> Vec<(String, DetailValue)> {
        let mut vec = Vec::new();
        // TODO add data to the vec
        vec
    }
}

#[update(name = "mint")]
#[candid_method(update)]
pub async fn mint(owner: Principal) {
    let ctx = get_context();
    let available = ctx.msg_cycles_available();
    let fee = 2000000000000;

    ic::print("avail:");
    ic::print(available.to_string());

    if available <= fee {
        panic!("Cannot mint less than {}", fee);
    }

    let accepted = ctx.msg_cycles_accept(available);

    let data = ic::get_mut::<Data>();

    let transaction_details = MintDetails {
        owner: owner,
        token_id: data.next_id,
        cycles: available,
    };

    data.nft_owners.insert(transaction_details.token_id, owner);

    data.next_id += data.next_id;

    let event = IndefiniteEventBuilder::new()
        .caller(ic::caller())
        .operation(String::from("mint"))
        .details(transaction_details)
        .build()
        .unwrap();

    insert(event).await.unwrap();
}

#[update(name = "transfer")]
#[candid_method(update)]
pub async fn transfer(new_owner: Principal, token_id: u64) {
    let ctx = get_context();
    let available = ctx.msg_cycles_available();
    let fee = 1000000000;

    if available <= fee {
        panic!("Cannot transfer less than {}", fee);
    }

    let accepted = ctx.msg_cycles_accept(available);
    let data = ic::get_mut::<Data>();
    let existing_owner = data.nft_owners.get(&token_id);

    let existing_owner = match data.nft_owners.get(&token_id) {
        Some(o) => o,
        None => {
            panic!("Error finding owner.");
        }
    };

    let caller = ic::caller();

    if caller != *existing_owner {
        panic!("Not owner.");
    }

    data.nft_owners.insert(token_id, new_owner);

    let transaction_details = TransferDetails {
        to: new_owner,
        token_id: token_id,
    };

    let event = IndefiniteEventBuilder::new()
        .caller(ic::caller())
        .operation(String::from("transfer"))
        .details(transaction_details)
        .build()
        .unwrap();

    insert(event).await.unwrap();
}

#[candid_method(update)]
#[update(name = "get_transaction_by_id")]
pub async fn get_transaction_by_id(id: u64) -> Event {
    let ctx = get_context();

    let result = get_transaction(id).await;

    let tx = match result {
        Ok(t) => t,
        Err(e) => {
            panic!("Error finding transactions.");
        }
    };

    tx
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
