use std::collections::HashMap;

use candid::{Nat, Principal};
use cap_sdk::{DetailValue, IntoDetails, TryFromDetails};
use serde::{Deserialize, Serialize};

pub struct XTCTransactionDetailsERC20 {
    to: Principal,
    amount: Nat,
    fee: Nat,
    index: Nat,
}

impl IntoDetails for XTCTransactionDetailsERC20 {
    fn into_details(self) -> Vec<(String, DetailValue)> {
        vec![
            ("to".into(), DetailValue::Principal(self.to)),
            ("amount".into(), self.amount.into()),
            ("fee".into(), self.fee.into()),
            ("index".into(), self.index.into()),
        ]
    }
}

impl TryFromDetails for XTCTransactionDetailsERC20 {
    fn try_from_details(details: &Vec<(String, DetailValue)>) -> Result<Self, ()> {
        let map = details.iter().cloned().collect::<HashMap<_, _>>();

        Ok(Self {
            to: map.get("principal").unwrap().clone().try_into()?,
            amount: map.get("amount").unwrap().clone().try_into()?,
            fee: map.get("fee").unwrap().clone().try_into()?,
            index: map.get("index").unwrap().clone().try_into()?,
        })
    }
}

pub struct XTCTransactionDetailsLegacy {
    fee: Nat,
    cycles: Nat,
    kind: XTCTransactionKindLegacy,
}

impl IntoDetails for XTCTransactionDetailsLegacy {
    fn into_details(self) -> Vec<(String, DetailValue)> {
        vec![
            ("fee".into(), self.fee.into()),
            ("cycles".into(), self.cycles.into()),
            (
                "kind".into(),
                DetailValue::Slice(bincode::serialize(&self.kind).unwrap()),
            ),
        ]
    }
}

impl TryFromDetails for XTCTransactionDetailsLegacy {
    fn try_from_details(details: &Vec<(String, DetailValue)>) -> Result<Self, ()> {
        let map = details.iter().cloned().collect::<HashMap<_, _>>();

        let kind = {
            if let Some(DetailValue::Slice(bytes)) = map.get("kind") {
                bincode::deserialize(bytes).map_err(|_| ())
            } else {
                Err(())
            }
        }?;

        Ok(Self {
            fee: map.get("fee").unwrap().clone().try_into()?,
            cycles: map.get("cycles").unwrap().clone().try_into()?,
            kind,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub enum XTCTransactionKindLegacy {
    Transfer {
        from: Principal,
        to: Principal,
    },

    Mint {
        to: Principal,
    },

    Burn {
        from: Principal,
        to: Principal,
    },

    CanisterCalled {
        from: Principal,
        to: Principal,
        method: String,
    },

    CanisterCreated {
        from: Principal,
        canister: Principal,
    },

    TransferFrom {
        from: Principal,
        to: Principal,
    },

    Approve {
        from: Principal,
        to: Principal,
    },
}
