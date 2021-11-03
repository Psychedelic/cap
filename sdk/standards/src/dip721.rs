use candid::Principal;
use cap_sdk::{TryFromEvent, IntoEvent};

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
    fn try_from_event(event: impl Into<cap_sdk::IndefiniteEvent>) -> Result<Self, ()> {
        let event = event.into();

        // let details = event.details;

        // let map = details.iter().cloned().collect::<HashMap<_, _>>();


        // Ok(match event.operation.as_str() {
        //     _ => return Err(())
        // })

        todo!() 
    }
}