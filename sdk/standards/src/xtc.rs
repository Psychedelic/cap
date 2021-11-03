use std::{collections::HashMap, convert::TryInto};

use candid::{Nat, Principal};
use cap_sdk::{DetailValue, IndefiniteEvent, IntoEvent, TryFromEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct XTCTransactionDetailsERC20 {
    to: Principal,
    amount: Nat,
    fee: Nat,
    index: Nat,
}

impl IntoEvent for XTCTransactionDetailsERC20 {
    fn details(&self) -> Vec<(String, DetailValue)> {
        vec![
            ("to".into(), self.to.into()),
            ("amount".into(), self.amount.clone().into()),
            ("fee".into(), self.fee.clone().into()),
            ("index".into(), self.index.clone().into()),
        ]
    }
}

impl TryFromEvent for XTCTransactionDetailsERC20 {
    fn try_from_event(event: impl Into<IndefiniteEvent>) -> Result<Self, ()> {
        let details = event.into().details;
        
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

impl IntoEvent for XTCTransactionKindLegacy {
    fn operation(&self) -> &'static str {
        match *self {
            Self::Transfer { .. } => "transfer",
            Self::TransferFrom { .. } => "transfer_from",
            Self::Approve { .. } => "approve",
            Self::Burn { .. } => "burn",
            Self::Mint { .. } => "mint",
            Self::CanisterCalled { .. } => "canister_called",
            Self::CanisterCreated { .. } => "canister_created"
        }
    }

    fn details(&self) -> Vec<(String, DetailValue)> {
        match self {
            Self::Transfer { from, to } => {
                vec![
                    ("to".to_owned(), to.clone().into()),
                    ("from".to_owned(), from.clone().into())
                ]
            },
            Self::TransferFrom { from, to } => {
                vec![
                    ("to".to_owned(), to.clone().into()),
                    ("from".to_owned(), from.clone().into())
                ]                
            },
            Self::Approve { from, to } => {
                vec![
                    ("to".to_owned(), to.clone().into()),
                    ("from".to_owned(), from.clone().into())
                ]
            },
            Self::Burn { from, to } => {
                vec![
                    ("to".to_owned(), to.clone().into()),
                    ("from".to_owned(), from.clone().into())
                ]
            },
            Self::Mint { to } => {
                vec![
                    ("to".to_owned(), to.clone().into())
                ]
            },
            Self::CanisterCalled { from, to, method } => {
                vec![
                    ("to".to_owned(), to.clone().into()),
                    ("from".to_owned(), from.clone().into()),
                    ("method".to_owned(), method.clone().into())
                ]
            },
            Self::CanisterCreated { from, canister } => {
                vec![
                    ("canister".to_owned(), canister.clone().into()),
                    ("from".to_owned(), from.clone().into())
                ]
            }
        }
    }
}

impl TryFromEvent for XTCTransactionKindLegacy {
    fn try_from_event(event: impl Into<IndefiniteEvent>) -> Result<Self, ()> {
        let event = event.into();
        let details = event.details;

        let map = details.iter().cloned().collect::<HashMap<_, _>>();


        Ok(match event.operation.as_str() {
            "transfer" => {
                XTCTransactionKindLegacy::Transfer {
                    to: map.get("to").unwrap().clone().try_into()?,
                    from: map.get("from").unwrap().clone().try_into()?
                }
            },
            "transfer_from" => {
                XTCTransactionKindLegacy::TransferFrom {
                    to: map.get("to").unwrap().clone().try_into()?,
                    from: map.get("from").unwrap().clone().try_into()?
                }
            },
            "approve" => {
                XTCTransactionKindLegacy::Approve {
                    to: map.get("to").unwrap().clone().try_into()?,
                    from: map.get("from").unwrap().clone().try_into()?
                }
            },
            "burn" => {
                XTCTransactionKindLegacy::Burn {
                    to: map.get("to").unwrap().clone().try_into()?,
                    from: map.get("from").unwrap().clone().try_into()?
                }
            },
            "mint" => {
                XTCTransactionKindLegacy::Mint {
                    to: map.get("to").unwrap().clone().try_into()?,
                }
            },
            "canister_called" => {
                XTCTransactionKindLegacy::CanisterCalled {
                    method: map.get("method").unwrap().clone().try_into()?,
                    to: map.get("to").unwrap().clone().try_into()?,
                    from: map.get("from").unwrap().clone().try_into()?
                }
            },
            "canister_created" => {
                XTCTransactionKindLegacy::CanisterCreated {
                    canister: map.get("canister").unwrap().clone().try_into()?,
                    from: map.get("from").unwrap().clone().try_into()?
                }
            }
            _ => return Err(())
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
