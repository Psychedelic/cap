use cap_core::{Bucket, GetContractRootError, Index, RootBucket, Router};
use ic_kit::ic::{get, get_maybe, store};
use ic_kit::{Context, Principal};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// Contains data about the cap environment.
///
/// Used by the SDK to perform operations transparently.
#[derive(Clone)]
pub struct CapEnv {
    pub(crate) root: RootBucket,
    pub(crate) index: Index,
}

impl CapEnv {
    pub(crate) async fn create(index: Principal) -> Result<CapEnv, GetContractRootError> {
        let index = Index::new(index);

        let root = index
            .get_token_contract_root_bucket(ic_kit::ic::id(), false)
            .await?;

        Ok(CapEnv { root, index })
    }

    pub(crate) fn store(&self) {
        store(self.clone());
    }

    pub(crate) fn get<'a>() -> &'a Self {
        if let Some(data) = get_maybe::<CapEnv>() {
            data
        } else {
            store(Self::create(Principal::from_str("TODO").unwrap()));

            // Unwrap here because we just stored the freshly created env
            // and if somehow `get_maybe` failed, there's a bigger issue
            // occurring.
            get_maybe::<CapEnv>().unwrap()
        }
    }

    // pub(crate) async fn get_mut<'a>() -> &'a mut Self {
    //     if let Some(data) = get_maybe::<CapEnv>() {
    //         data
    //     } else {
    //         store(Self::create(Principal::from_str("TODO").unwrap()));
    //
    //         // Unwrap here because we just stored the freshly created env
    //         // and if somehow `get_maybe` failed, there's a bigger issue
    //         // occurring.
    //         get_maybe::<CapEnv>().unwrap()
    //     }
    // }

    /// Sets the [`CapEnv`] using the provided value.
    ///
    /// Used to restore the generated canister's ID after an upgrade.
    pub fn load_from_archive(env: CapEnv) {
        env.store();
    }

    /// Gets the [`CapEnv`].
    ///
    /// Should be used during the upgrade process of a contract canister.
    /// Call it during `pre_upgrade` to write it somewhere in stable storage.
    ///
    /// Afterwards, write it back with [`CapEnv::load_from_archive`]
    pub fn to_archive() -> Self {
        CapEnv::get().clone()
    }
}
