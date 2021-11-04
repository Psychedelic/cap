use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use candid::{Int, Nat, Principal};
use cap_sdk::{DetailValue, IntoEvent, MaybeIndefinite, TryFromEvent};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub struct TxDetails {
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

#[cfg(feature = "sdk-impls")]
impl IntoEvent for TxDetails {
    fn operation(&self) -> &'static str {
        self.operation.into()
    }

    fn details(&self) -> Vec<(String, DetailValue)> {
        let status: &'static str = self.status.into();

        vec![
            ("index".into(), self.index.clone().into()),
            ("from".into(), self.from.into()),
            ("to".into(), self.to.into()),
            ("amount".into(), self.amount.clone().into()),
            ("fee".into(), self.fee.clone().into()),
            ("timestamp".into(), self.to.into()),
            ("status".into(), status.to_owned().into()),
        ]
    }
}

#[cfg(feature = "sdk-impls")]
trait MapFailed<T, E> {
    fn map_failure(self, key: &'static str) -> Result<T, E>;
}

#[cfg(feature = "sdk-impls")]
impl<T, O> MapFailed<T, DIP20ParseError> for Result<T, O> {
    fn map_failure(self, key: &'static str) -> Result<T, DIP20ParseError> {
        self.map_err(|_| DIP20ParseError::ConversionError(key.to_owned()))
    }
}

#[cfg(feature = "sdk-impls")]
fn try_get_and_clone(
    map: &HashMap<String, DetailValue>,
    key: &'static str,
) -> Result<DetailValue, DIP20ParseError> {
    if let Some(item) = map.get(key) {
        Ok(item.clone())
    } else {
        Err(DIP20ParseError::MissingKey(key.to_owned()))
    }
}

#[cfg(feature = "sdk-impls")]
impl TryFromEvent for TxDetails {
    type Error = DIP20ParseError;

    fn try_from_event(event: impl MaybeIndefinite) -> Result<Self, DIP20ParseError> {
        let timestamp = if let Some(time) = event.time() {
            Int(BigInt::default() + time)
        } else {
            return Err(DIP20ParseError::MissingKey("time".to_owned()));
        };

        let event = event.as_indefinite();

        let details = event.details;

        let map = details.iter().cloned().collect::<HashMap<_, _>>();

        let status_string: String = try_get_and_clone(&map, "status")?
            .try_into()
            .map_failure("status")?;

        Ok(Self {
            index: try_get_and_clone(&map, "index")?
                .try_into()
                .map_failure("index")?,
            from: try_get_and_clone(&map, "from")?
                .try_into()
                .map_failure("from")?,
            to: try_get_and_clone(&map, "to")?
                .try_into()
                .map_failure("to")?,
            amount: try_get_and_clone(&map, "amount")?
                .try_into()
                .map_failure("amount")?,
            fee: try_get_and_clone(&map, "fee")?
                .try_into()
                .map_failure("fee")?,
            timestamp,
            status: status_string.as_str().try_into().map_err(|_| {
                DIP20ParseError::ConversionError("Invalid value for status".to_owned())
            })?,
            operation: event
                .operation
                .as_str()
                .try_into()
                .map_err(|_| DIP20ParseError::InvalidOperation(String::from("")))?,
        })
    }
}
