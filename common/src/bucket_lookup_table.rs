use crate::did::TransactionId;
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};

/// A data structure to store a linear list of buckets, each bucket has a starting offset which
/// is a transaction id and determine the starting range of transactions that this bucket contains,
/// this data structure can be used to answer the question: "Which bucket contains transaction N?"
/// and also issue a witness proving the result.
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
    /// Insert a new bucket to the list of buckets.
    ///
    /// # Panics
    /// If the provided transaction id is not larger than the previous starting offset.
    #[inline(always)]
    pub fn insert(&mut self, starting_offset: TransactionId, canister: Principal) {
        if !self.data.is_empty() {
            let ending_offset = self.data[self.data.len() - 1].0;
            assert!(starting_offset > ending_offset, "Invalid starting offset.");
        }

        let mut h = Sha256::new();
        h.update(canister.as_slice());
        let hash = h.finalize().into();
        self.data.push((starting_offset, canister));
        self.certified_map.insert(starting_offset.into(), hash);
    }

    /// Remove the last bucket from the list, and return the data that was associated with it.
    pub fn pop(&mut self) -> Option<(TransactionId, Principal)> {
        let data = self.data.pop();

        if let Some((id, _)) = &data {
            let id = TransactionIdKey::from(*id);
            self.certified_map.delete(id.as_ref());
        }

        data
    }

    /// Return the bucket that should contain the given offset.
    ///
    /// # Panics
    /// If the offset provided is smaller than the smallest offset in the buckets. This implies
    /// that this method will also panic if there are no buckets inserted yet.
    #[inline]
    pub fn get_bucket_for(&self, offset: TransactionId) -> &Principal {
        match self.data.binary_search_by(|probe| probe.0.cmp(&offset)) {
            Ok(index) => &self.data[index].1,
            Err(0) => panic!("Given offset is smaller than the starting offset of the chain."),
            Err(index) => &self.data[index - 1].1,
        }
    }

    /// Generate the HashTree witness for that proves the result returned from `get_bucket_for`
    /// method.
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

impl Serialize for BucketLookupTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.data.len()))?;
        for i in &self.data {
            s.serialize_element(i)?;
        }
        s.end()
    }
}

// TODO(qti3e) Test
