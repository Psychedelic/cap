use ic_certified_map::HashTree::Leaf;
use ic_certified_map::{leaf_hash, AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;

/// A data structure that maps a canister id to another canister id and
#[derive(Default)]
pub struct CanisterMap {
    data: RbTree<Principal, PrincipalBytes>,
}

struct PrincipalBytes(Principal);

impl From<Principal> for PrincipalBytes {
    #[inline]
    fn from(p: Principal) -> Self {
        Self(p)
    }
}

impl AsHashTree for PrincipalBytes {
    #[inline]
    fn root_hash(&self) -> Hash {
        leaf_hash(&self.0.as_ref())
    }

    #[inline]
    fn as_hash_tree(&self) -> HashTree<'_> {
        Leaf(&self.0.as_ref())
    }
}

impl CanisterMap {
    /// Insert the given relation into the map.
    #[inline]
    pub fn insert(&mut self, key: Principal, value: Principal) {
        self.data.insert(key, value.into());
    }

    /// Return the principal id associated with the given principal id.
    #[inline]
    pub fn get(&self, key: &Principal) -> Option<&Principal> {
        match self.data.get(key.as_ref()) {
            Some(bytes) => Some(&bytes.0),
            None => None,
        }
    }

    /// Create a HashTree witness for the value associated with the given key.
    #[inline]
    pub fn gen_witness(&self, key: &Principal) -> HashTree {
        self.data.witness(key.as_ref())
    }
}

impl AsHashTree for CanisterMap {
    fn root_hash(&self) -> Hash {
        self.data.root_hash()
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        self.data.as_hash_tree()
    }
}
