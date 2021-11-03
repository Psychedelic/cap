use std::collections::HashMap;

use candid::Principal;
use cap_sdk::{DetailValue, IntoEvent, TryFromEvent};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum DIP721TransactionType {
    TransferFrom(TransferFrom),
    Approve(Approve),
    SetApprovalForAll(SetApprovalForAll),
    Mint(Mint),
    Burn(Burn)
}

#[derive(Debug, Clone, Copy)]
pub struct TransferFrom {
    pub token_id: u64,
    pub from: Principal,
    pub to: Principal,
    pub caller: Option<Principal>
}

#[derive(Debug, Clone, Copy)]
pub struct Approve {
    pub token_id: u64,
    pub from: Principal,
    pub to: Principal
}

#[derive(Debug, Clone, Copy)]
pub struct SetApprovalForAll {
    pub from: Principal,
    pub to: Principal
}

#[derive(Debug, Clone, Copy)]
pub struct Mint {
    pub token_id: u64
}

#[derive(Debug, Clone, Copy)]
pub struct Burn {
    pub token_id: u64
}

impl IntoEvent for DIP721TransactionType {
    fn operation(&self) -> &'static str {
        match *self {
            Self::TransferFrom(_) => "transfer_from",
            Self::Approve(_) => "approve",
            Self::SetApprovalForAll(_) => "set_approval_for_all",
            Self::Mint(_) => "mint",
            Self::Burn(_) => "burn"
        }
    }
    
    fn details(&self) -> Vec<(String, cap_sdk::DetailValue)> {
        match *self {
            Self::TransferFrom(transfer) => {
                let mut data = vec![
                    ("token_id".to_owned(), transfer.token_id.into()),
                    ("from".to_owned(), transfer.from.into()),
                    ("to".to_owned(), transfer.to.into()),
                ];

                if let Some(caller) = transfer.caller {
                    data.push(("caller".to_owned(), caller.into()));
                }

                data
            },
            Self::Approve(approve) => {
                vec![
                    ("token_id".to_owned(), approve.token_id.into()),
                    ("from".to_owned(), approve.from.into()),
                    ("to".to_owned(), approve.to.into()),
                ]
            },
            Self::SetApprovalForAll(set_approval) => {
            vec![
                ("from".to_owned(), set_approval.from.into()),
                ("to".to_owned(), set_approval.to.into()),
            ]
            },
            Self::Mint(mint) => {
                vec![
                    ("token_id".to_owned(), mint.token_id.into()),
                ]
            },
            Self::Burn(burn) => {
                vec![
                ("token_id".to_owned(), burn.token_id.into()) 
            ]
            }
        }
    }
}

impl TryFromEvent for DIP721TransactionType {
    type Error = ();

    fn try_from_event(event: impl Into<cap_sdk::IndefiniteEvent>) -> Result<Self, ()> {

        todo!() 
    }
}



#[derive(Debug, Error)]
pub enum DIP721TransactionDecodeError {
    #[error("missing key {0}")]
    MissingKey(String),
    #[error("couldn't convert item with key {0} to DetailValue")]
    ConversionError(String),
    #[error("invalid operation {0}")]
    InvalidOperation(String )
}

trait MapFailed<T,E> {
    fn map_failure(self, key: &'static str) -> Result<T, E>;
}

impl<T, O> MapFailed<T, DIP721TransactionDecodeError> for Result<T, O> {
    fn map_failure(self, key: &'static str) -> Result<T, DIP721TransactionDecodeError> {
        self.map_err(|_| {
            DIP721TransactionDecodeError::ConversionError(key.to_owned())
        })
    }
}

fn try_get_and_clone(map: &HashMap<String, DetailValue>, key: &'static str) -> Result<DetailValue, DIP721TransactionDecodeError> {
    if let Some(item) = map.get(key) {
        Ok(item.clone())
    } else {
        Err(DIP721TransactionDecodeError::MissingKey(key.to_owned()))
    }
}