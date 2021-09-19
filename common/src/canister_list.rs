use crate::readable::ReadableCanisterId;
use ic_certified_map::{AsHashTree, Hash, HashTree};
use sha2::{Digest, Sha256};

/// An array of Canister IDs that can be certified.
#[derive(Default)]
pub struct CanisterList {
    data: Vec<ReadableCanisterId>,
    hash: Hash,
}

impl CanisterList {
    #[inline]
    pub fn push(&mut self, id: ReadableCanisterId) {
        let mut h = Sha256::new();
        h.update(&self.hash);
        h.update(id.as_slice());
        self.hash = h.finalize().into();
        self.data.push(id);
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[ReadableCanisterId] {
        self.data.as_slice()
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

// TODO(qti3e) Test
