use crate::context::CapContext;
use ic_kit::ic;
use ic_kit::Principal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

/// Number of new principal ids that once passed, we try to perform
/// another flush.
const USERS_FLUSH_THRESHOLD_COUNT: usize = 15;
/// Time that once passed since the last flush, we perform another
/// flush.
const USERS_FLUSH_THRESHOLD_TIME: u64 = 2000;
/// The minimum time that must be passed since the last flush in
/// order to perform another flush.
/// In other words if the last flush is performed less than x-ms
/// ago we should prevent a new flush.
/// This ensures that we only perform at most 1 call per each x-ms.
const USER_FLUSH_MIN_PASSED_TIME: u64 = 1000;
/// When rate limiting is enabled on the canister limit the number
/// of unique principal ids that each event can have to this number,
/// this limit should be disregarded for when [`CapContext::ignore_rate_limit`]
/// is set to `true`.
const MAX_PRINCIPALS_PER_EVENT: usize = 7;

#[derive(Default, Serialize, Deserialize)]
pub struct Users {
    seen: HashSet<Principal>,
    to_flush: HashSet<Principal>,
    last_flush: u64,
}

impl Users {
    /// Extract new user principal ids from the event in order to notify
    /// the root canister so that it can list us as interacted contract
    /// for the user.
    #[inline]
    pub fn insert(&mut self, ctx: &CapContext, principals: BTreeSet<&Principal>) {
        if !ctx.ignore_rate_limit && principals.len() > MAX_PRINCIPALS_PER_EVENT {
            panic!(
                "Number of principals in a single event can not exceed {}",
                MAX_PRINCIPALS_PER_EVENT
            );
        }

        for principal in principals {
            if self.seen.insert(*principal) {
                self.to_flush.insert(*principal);
            }
        }
    }

    /// Try to write the data to cap router if the flush conditions are met, otherwise
    /// it's just a no-op.
    #[inline]
    pub fn trigger_flush(&mut self, ctx: &'static CapContext) {
        if self.to_flush.is_empty() {
            return;
        }

        // div by 1e6 to convert from nanoseconds to milliseconds.
        let time = ic::time() / 1_000_000;

        // I'm not sure if you can travel back in time on IC when dealing with time
        // in ms, so it's better than be safe than sorry and use saturating_sub
        // here.
        let time_passed_since_last_flush = time.saturating_sub(self.last_flush);

        // This ensures that we perform at most 1 request to the router canister
        // second.
        if time_passed_since_last_flush < USER_FLUSH_MIN_PASSED_TIME {
            return;
        }

        if self.to_flush.len() >= USERS_FLUSH_THRESHOLD_COUNT
            || time_passed_since_last_flush >= USERS_FLUSH_THRESHOLD_TIME
        {
            let principals = self.to_flush.drain().collect::<Vec<_>>();
            ic_cdk::block_on(perform_flush(ctx, principals));
            self.last_flush = time;
        }
    }
}

/// Tries to write the new unique principal ids that have interacted with this
/// token contract to Cap's router, we do this in a fail tolerant way.
async fn perform_flush(ctx: &'static CapContext, principals: Vec<Principal>) {
    let cap_id = ctx.cap_canister_id;
    let contract_id = &ctx.contract_id;

    // Retry 3 times, if failed after all retries try to insert the
    // transactions back to Users structure.
    for _ in 0..3 {
        let args = (contract_id, &principals);
        if ic::call::<(&Principal, &Vec<Principal>), (), &str>(cap_id, "insert_new_users", args)
            .await
            .is_ok()
        {
            return;
        }
    }

    // This case is really unlikely to happen in reality, the only possibilities
    // that I reckon this case being met is for when there is a network mis-
    // configuration on the current subnet, so that it can not reach the
    // subnet in which Cap's router canister is located, so it's worth to
    // take that kind of possibility into account to be more fail tolerant.
    // In this case there is nothing we can really do that not losing the
    // data so we can insert it once the subnet issue is resolved.

    let users = ic::get_mut::<Users>();
    users.to_flush.extend(principals.into_iter());
}
