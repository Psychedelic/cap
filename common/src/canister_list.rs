use ic_certified_map::{AsHashTree, Hash, HashTree};
use ic_kit::Principal;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};

/// An array of Canister IDs with incremental hashing, this can be used as a leaf node in a
/// certified RbTree.
#[derive(Default)]
pub struct CanisterList {
    data: Vec<Principal>,
    hash: Hash,
}

impl CanisterList {
    /// Insert the given principal id to the list, and update the hash.
    #[inline]
    pub fn push(&mut self, id: Principal) {
        let mut h = Sha256::new();
        h.update(&self.hash);
        h.update(id.as_slice());
        self.hash = h.finalize().into();
        self.data.push(id);
    }

    /// Return the list as slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[Principal] {
        self.data.as_slice()
    }

    /// Return the list as a vector.
    #[inline(always)]
    pub fn to_vec(&self) -> Vec<Principal> {
        self.data.clone()
    }
}

impl AsHashTree for CanisterList {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        self.hash.root_hash()
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree<'_> {
        self.hash.as_hash_tree()
    }
}

impl Serialize for CanisterList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.data.len()))?;
        for principal in &self.data {
            s.serialize_element(principal)?;
        }
        s.end()
    }
}

// TODO(qti3e) Test
