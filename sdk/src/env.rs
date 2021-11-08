use cap_sdk_core::{Index, RootBucket, Router};
use futures::future::LocalBoxFuture;
use ic_kit::ic::{get_maybe, store};
use std::cell::RefCell;

/// Contains data about the cap environment.
#[derive(Clone)]
pub struct CapEnv {
    pub(crate) root: RootBucket,
    pub(crate) router: Router,
}

thread_local! {
    pub(crate) static FUTURES: RefCell<Vec<LocalBoxFuture<'static, ()>>> = RefCell::new(vec![]);
}

impl CapEnv {
    /// Creates a new [`CapEnv`] with the index canister's [`Principal`] set to `index`.
    pub(crate) fn create(root: RootBucket, router: Router) -> Self {
        CapEnv { root, router }
    }

    /// Stores the [`CapEnv`] in the canister.
    pub(crate) fn store(&self) {
        store(self.clone());
    }

    pub(crate) fn get<'a>() -> &'a Self {
        if let Some(data) = get_maybe::<CapEnv>() {
            data
        } else {
            panic!("No context created.");
        }
    }

    pub(crate) fn index(&self) -> Index {
        self.router.into()
    }

    pub(crate) async fn await_futures() {
        let futures = FUTURES.with(|futures| {
            let mut inner = futures.take();

            inner.drain(0..inner.len()).collect::<Vec<_>>()
        });

        for future in futures {
            future.await;
        }
    }

    pub(crate) fn insert_future(future: LocalBoxFuture<'static, ()>) {
        FUTURES.with(|futures| {
            futures.borrow_mut().push(future);
        });
    }

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
