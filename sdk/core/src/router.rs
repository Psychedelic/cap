//! Contains logic for interacting with cap's main router.
//!
//! For more information on the purpose of the main router, see the documentation on
//! [`Router`].

use ic_kit::Principal;

/// A router.
///
/// The main router is an extended version of [`Index`] which allows registration of a new
/// contract to cap. It also contains global index canister registry which allows querying
/// for the root bucket of a contract.
///
/// A router bucket implements the same interface as [`Index`], but with 1 additional method.
///
/// Use [`Router`]'s [`Into<Index>`] implementation to use a [`Router`] as an [`Index`].
pub struct Router(pub(crate) Principal);

// TODO
