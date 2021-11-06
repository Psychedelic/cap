use ic_kit::ic;
use ic_kit::Principal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The general meta information that we store for this root bucket
/// canister.
///
/// This is a singleton struct.
#[derive(Clone, Serialize, Deserialize)]
pub struct CapContext {
    /// Principal id of the main cap's router canister.
    pub cap_canister_id: Principal,
    /// Principal id of the contract that is writing its history
    /// to this root bucket.
    pub contract_id: Principal,
    /// The principal id of the different canisters or user pid that
    /// are allowed to write information and insert transactions to
    /// this root bucket.
    pub writers: HashSet<Principal>,
    /// If set to be true, we skip any rate limiting applied to the
    /// insertions, by default this value should be set to false for
    /// newly generated buckets and we should only change it to true
    /// if a request goes thought the governance module from the
    /// router.
    ///
    /// The rate limiting is used to prevent DDoS attack on the router
    /// canister.
    pub ignore_rate_limit: bool,
}

impl CapContext {
    /// Returns the context used to communicate with Cap.
    ///
    /// # Panics
    ///
    /// If executed before setting up the context, the default
    /// set up happens during init.
    pub fn get() -> &'static CapContext {
        ic::get_maybe::<CapContext>().unwrap()
    }

    /// Returns true if the given principal id is allowed to insert
    /// transactions in this bucket.
    pub fn is_writer(&self, pid: &Principal) -> bool {
        if pid == &self.contract_id {
            return true;
        }

        if self.writers.contains(pid) {
            return true;
        }

        false
    }

    /// Change the ignore_rate_limit to true to enable bypassing any
    /// rate limits.
    pub fn bypass_rate_limit() {
        let mut data = Self::get().clone();
        data.ignore_rate_limit = true;
        ic::store(data);
    }
}
