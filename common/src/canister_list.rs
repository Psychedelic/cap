use ic_certified_map::{AsHashTree, Hash, HashTree};
use ic_kit::Principal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// An array of Canister IDs with incremental hashing, this can be used as a leaf node in a
/// certified RbTree.
#[derive(Deserialize, Serialize)]
pub struct CanisterList {
    data: Vec<Principal>,
    hash: Hash,
}

impl CanisterList {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            hash: [0; 32],
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use ic_kit::mock_principals;

    #[test]
    fn push() {
        let mut list = CanisterList::new();
        assert_eq!(list.hash, [0; 32]);

        list.push(mock_principals::alice());
        let hash1 = list.hash;

        list.push(mock_principals::bob());
        let hash2 = list.hash;

        assert_ne!(hash1, hash2);
        assert_eq!(
            list.to_vec(),
            vec![mock_principals::alice(), mock_principals::bob()]
        );
    }
}
