use std::{collections::HashMap, convert::TryInto};

use candid::{Int, Nat, Principal};
use cap_sdk::{DetailValue, DetailsBuilder, IndefiniteEvent, IntoEvent, TryFromEvent, TypedEvent};
use num_bigint::{BigInt, BigUint};

use super::{DIP20ParseError, Operation, TransactionStatus, TxRecord};

#[derive(Clone)]
pub enum DIP20Details {
    Approve {
        from: Principal,
        to: Principal,
        amount: Nat,
        fee: Nat,
        status: TransactionStatus,
    },
    Mint {
        from: Principal,
        to: Principal,
        amount: Nat,
        fee: Nat,
        status: TransactionStatus,
    },
    Transfer {
        from: Principal,
        to: Principal,
        amount: Nat,
        fee: Nat,
        status: TransactionStatus,
    },
    TransferFrom {
        from: Principal,
        to: Principal,
        amount: Nat,
        fee: Nat,
        status: TransactionStatus,
    },
}

pub trait DIP20EventExt {
    fn timestamp(&self) -> Int;
    fn caller(&self) -> Principal;

    fn into_txrecord(self) -> TxRecord;
}

impl DIP20EventExt for TypedEvent<DIP20Details> {
    fn timestamp(&self) -> Int {
        Int(BigInt::default() + self.time)
    }

    fn caller(&self) -> Principal {
        self.caller
    }

    fn into_txrecord(self) -> TxRecord {
        match self.details {
            DIP20Details::Approve {
                from,
                to,
                amount,
                fee,
                status,
            } => TxRecord {
                caller: self.caller,
                timestamp: Int(BigInt::default() + self.time),
                index: Nat(BigUint::default()),
                from,
                to,
                amount,
                fee,
                status,
                operation: Operation::Approve,
            },
            DIP20Details::Mint {
                from,
                to,
                amount,
                fee,
                status,
            } => TxRecord {
                caller: self.caller,
                timestamp: Int(BigInt::default() + self.time),
                index: Nat(BigUint::default()),
                from,
                to,
                amount,
                fee,
                status,
                operation: Operation::Mint,
            },
            DIP20Details::Transfer {
                from,
                to,
                amount,
                fee,
                status,
            } => TxRecord {
                caller: self.caller,
                timestamp: Int(BigInt::default() + self.time),
                index: Nat(BigUint::default()),
                from,
                to,
                amount,
                fee,
                status,
                operation: Operation::Transfer,
            },
            DIP20Details::TransferFrom {
                from,
                to,
                amount,
                fee,
                status,
            } => TxRecord {
                caller: self.caller,
                timestamp: Int(BigInt::default() + self.time),
                index: Nat(BigUint::default()),
                from,
                to,
                amount,
                fee,
                status,
                operation: Operation::TransferFrom,
            },
        }
    }
}

impl IntoEvent for DIP20Details {
    fn operation(&self) -> &'static str {
        match self {
            Self::Approve { .. } => "approve",
            Self::Mint { .. } => "mint",
            Self::Transfer { .. } => "transfer",
            Self::TransferFrom { .. } => "transfer_from",
        }
    }

    fn details(&self) -> Vec<(String, cap_sdk::DetailValue)> {
        match self {
            Self::Approve {
                from,
                to,
                amount,
                fee,
                status,
            } => {
                let status_string = status.into_str().to_owned();

                DetailsBuilder::default()
                    .insert("from", from.clone())
                    .insert("to", to.clone())
                    .insert("amount", amount.clone())
                    .insert("fee", fee.clone())
                    .insert("status", status_string.clone())
                    .build()
            }
            Self::Mint {
                from,
                to,
                amount,
                fee,
                status,
            } => {
                let status_string = status.into_str().to_owned();

                DetailsBuilder::default()
                    .insert("from", from.clone())
                    .insert("to", to.clone())
                    .insert("amount", amount.clone())
                    .insert("fee", fee.clone())
                    .insert("status", status_string.clone())
                    .build()
            }
            Self::Transfer {
                from,
                to,
                amount,
                fee,
                status,
            } => {
                let status_string = status.into_str().to_owned();

                DetailsBuilder::default()
                    .insert("from", from.clone())
                    .insert("to", to.clone())
                    .insert("amount", amount.clone())
                    .insert("fee", fee.clone())
                    .insert("status", status_string.clone())
                    .build()
            }
            Self::TransferFrom {
                from,
                to,
                amount,
                fee,
                status,
            } => {
                let status_string = status.into_str().to_owned();

                DetailsBuilder::default()
                    .insert("from", from.clone())
                    .insert("to", to.clone())
                    .insert("amount", amount.clone())
                    .insert("fee", fee.clone())
                    .insert("status", status_string.clone())
                    .build()
            }
        }
    }
}

impl TryFromEvent for DIP20Details {
    type Error = DIP20ParseError;

    fn try_from_event(event: impl Into<IndefiniteEvent>) -> Result<Self, Self::Error> {
        let event = event.into();

        let details = event.details.iter().cloned().collect::<HashMap<_, _>>();

        match event.operation.as_str() {
            "approve" => {
                let status_string: String = details
                    .get_detail("status")?
                    .try_into()
                    .map_failure("status")?;

                Self::Approve {
                    from: details.get_detail("from")?.try_into().map_failure("from")?,
                    to: details.get_detail("to")?.try_into().map_failure("to")?,
                    amount: details
                        .get_detail("amount")?
                        .try_into()
                        .map_failure("amount")?,
                    fee: details.get_detail("fee")?.try_into().map_failure("fee")?,
                    status: status_string.as_str().try_into().map_failure("status")?,
                };
            }
            "mint" => {
                let status_string: String = details
                    .get_detail("status")?
                    .try_into()
                    .map_failure("status")?;

                Self::Mint {
                    from: details.get_detail("from")?.try_into().map_failure("from")?,
                    to: details.get_detail("to")?.try_into().map_failure("to")?,
                    amount: details
                        .get_detail("amount")?
                        .try_into()
                        .map_failure("amount")?,
                    fee: details.get_detail("fee")?.try_into().map_failure("fee")?,
                    status: status_string.as_str().try_into().map_failure("status")?,
                };
            }
            "transfer" => {
                let status_string: String = details
                    .get_detail("status")?
                    .try_into()
                    .map_failure("status")?;

                Self::Transfer {
                    from: details.get_detail("from")?.try_into().map_failure("from")?,
                    to: details.get_detail("to")?.try_into().map_failure("to")?,
                    amount: details
                        .get_detail("amount")?
                        .try_into()
                        .map_failure("amount")?,
                    fee: details.get_detail("fee")?.try_into().map_failure("fee")?,
                    status: status_string.as_str().try_into().map_failure("status")?,
                };
            }
            "transfer_from" => {
                let status_string: String = details
                    .get_detail("status")?
                    .try_into()
                    .map_failure("status")?;

                Self::TransferFrom {
                    from: details.get_detail("from")?.try_into().map_failure("from")?,
                    to: details.get_detail("to")?.try_into().map_failure("to")?,
                    amount: details
                        .get_detail("amount")?
                        .try_into()
                        .map_failure("amount")?,
                    fee: details.get_detail("fee")?.try_into().map_failure("fee")?,
                    status: status_string.as_str().try_into().map_failure("status")?,
                };
            }
            _ => {}
        }

        todo!()
    }
}

trait GetDetailExt {
    fn get_detail(&self, detail: &'static str) -> Result<DetailValue, DIP20ParseError>;
}

impl GetDetailExt for HashMap<String, DetailValue> {
    fn get_detail(&self, detail: &'static str) -> Result<DetailValue, DIP20ParseError> {
        if let Some(detail) = self.get(detail) {
            Ok(detail.clone())
        } else {
            Err(DIP20ParseError::MissingKey(detail.to_owned()))
        }
    }
}

trait MapFailed<T, E> {
    fn map_failure(self, key: &'static str) -> Result<T, E>;
}

impl<T, O> MapFailed<T, DIP20ParseError> for Result<T, O> {
    fn map_failure(self, key: &'static str) -> Result<T, DIP20ParseError> {
        self.map_err(|_| DIP20ParseError::ConversionError(key.to_owned()))
    }
}
