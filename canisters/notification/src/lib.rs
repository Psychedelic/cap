use std::{collections::{HashMap, HashSet}, hash::Hash};
use ic_cdk_macros::{post_upgrade, pre_upgrade, query, update};
use ic_cdk::{print, storage};
use std::cell::RefCell;
use serde::{Deserialize, Serialize};
use ic_cdk::export::candid::{CandidType, Principal};

type TargetCanister = Principal;
type Subscriber = HashMap<TargetCanister, HashSet<ProxyEvent>>;

#[derive(Hash, Clone, Debug, CandidType, Deserialize)]
struct MessagePayload(Vec<u8>);

#[derive(Debug)]
pub struct ProxyEvent {
    /// the source token pid of the message
    token: Principal,
    /// a generic message payload
    message: MessagePayload,
    /// method name of `destination_canister` where they expect a publish
    method_name: String,
}   

impl ProxyEvent {
    pub fn new(token: Principal, message: MessagePayload, method_name: String) -> Self {
        Self { token, message, method_name }
    }

    pub fn token(&self) -> &Principal {
        &self.token
    }

    pub fn message(&self) -> &MessagePayload {
        &self.message
    }

    pub fn method_name(&self) -> &str {
        &self.method_name
    }
}

#[derive(Deserialize, CandidType)]
pub enum SuccessMessage {
    Subbed,
    UnSubbed,
}

#[derive(Deserialize, CandidType)]
pub enum ErrorMessage {
    /// Internal error indicading canister level issues with publishing
    PublishError,
    /// Internal error indicading canister level issues with subscribing
    SubscribeError,
    /// Internal canister error
    CanisterError{ code: i32, msg: String },
    /// Common unknown error
    Unknown
}

#[derive(Deserialize, CandidType)]
pub enum Response {
    /// Call was successful
    Success(SuccessMessage), 
    /// Call errored
    Error(ErrorMessage),
}

#[derive(Default)]
pub struct SubscriberFactory {
    subscribers: RefCell<Subscriber>,
}

#[derive(Default)]
struct StableSubscribers {
    subscribers: Subscriber,
}

impl SubscriberFactory {
    pub fn new(subscribers: SubscriberFactory) -> Self {
        SubscriberFactory {
            subscribers: HashMap::new()
        }
    }

    pub fn subscribe() {
        todo!()
    }

    pub fn unsubscribe() {
        todo!()
    }

    pub fn take_all(&self) -> StableSubscribers {
        StableSubscribers {
            subscribers: storage::get_mut::<SubscriberFactory>().subscribers.take()
        }
    }

    pub fn clear_all(&self) {
        self.subscribers.borrow_mut().clear();
    }

    pub fn replace_all(&self, stable_notify: StableSubscribers) {
        self.subscribers.replace(stable_notify.subscribers);
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    let stable_services = storage::get::<SubscriberFactory>().take_all();
    storage::stable_save((stable_services,)).expect("failed to save stable notifications");
}

#[post_upgrade]
fn post_upgrade() {
    storage::get::<SubscriberFactory>().clear_all();

    let (stable_services,): (StableNotifications,) =
        storage::stable_restore().expect("failed to restore stable notifications");

    storage::get::<Notifications>().replace_all(stable_services);
}
