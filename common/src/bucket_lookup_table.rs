use crate::readable::TransactionId;
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct BucketLookupTable {
    data: Vec<(TransactionId, Principal)>,
    certified_map: RbTree<TransactionIdKey, Hash>,
}

struct TransactionIdKey([u8; 8]);

impl From<TransactionId> for TransactionIdKey {
    #[inline(always)]
    fn from(n: TransactionId) -> Self {
        TransactionIdKey(n.to_be_bytes())
    }
}

impl AsRef<[u8]> for TransactionIdKey {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl BucketLookupTable {
    #[inline(always)]
    pub fn insert(&mut self, starting_offset: TransactionId, canister: Principal) {
        let mut h = Sha256::new();
        h.update(canister.as_slice());
        let hash = h.finalize().into();
        self.data.push((starting_offset, canister));
        self.certified_map.insert(starting_offset.into(), hash);
    }

    #[inline]
    pub fn get_bucket_for(&self, offset: TransactionId) -> &Principal {
        match self.data.binary_search_by(|probe| probe.0.cmp(&offset)) {
            Ok(index) => &self.data[index].1,
            Err(index) => &self.data[index - 1].1,
        }
    }

    #[inline]
    pub fn gen_witness(&self, offset: TransactionId) -> HashTree {
        self.certified_map
            .witness(TransactionIdKey::from(offset).as_ref())
    }
}

impl AsHashTree for BucketLookupTable {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        self.certified_map.root_hash()
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree<'_> {
        self.certified_map.as_hash_tree()
    }
}

// TODO(qti3e) Test
