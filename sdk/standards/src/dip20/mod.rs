use std::{
    convert::TryFrom,
};

use candid::{Int, Nat, Principal};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "sdk-impls")]
mod cap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransactionStatus {
    Succeeded,
    Failed,
}

impl<'a> TryFrom<&'a str> for TransactionStatus {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            _ => return Err(()),
        })
    }
}

impl Into<&'static str> for TransactionStatus {
    fn into(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

impl TransactionStatus {
    pub fn into_str(self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Operation {
    Approve,
    Mint,
    Transfer,
    TransferFrom,
}

impl<'a> TryFrom<&'a str> for Operation {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "approve" => Self::Approve,
            "mint" => Self::Mint,
            "transfer" => Self::Transfer,
            "transfer_from" => Self::TransferFrom,
            _ => return Err(()),
        })
    }
}

impl Into<&'static str> for Operation {
    fn into(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Mint => "mint",
            Self::Transfer => "transfer",
            Self::TransferFrom => "transfer_from",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TxRecord {
    pub caller: Principal,
    pub index: Nat,
    pub from: Principal,
    pub to: Principal,
    pub amount: Nat,
    pub fee: Nat,
    pub timestamp: Int,
    pub status: TransactionStatus,
    pub operation: Operation,
}

#[derive(Debug, Error)]
pub enum DIP20ParseError {
    #[error("missing key {0}")]
    MissingKey(String),
    #[error("couldn't convert item with key {0} to DetailValue")]
    ConversionError(String),
    #[error("invalid operation {0}")]
    InvalidOperation(String),
}
