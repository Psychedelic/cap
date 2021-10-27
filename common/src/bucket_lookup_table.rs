use crate::did::TransactionId;
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BucketLookupTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type T = Vec<(TransactionId, Principal)>;
        let data = T::deserialize(deserializer)?;
        let mut certified_map = RbTree::new();

        for (id, principal) in &data {
            let mut h = Sha256::new();
            h.update(principal.as_slice());
            let hash = h.finalize().into();

            certified_map.insert(TransactionIdKey::from(*id), hash);
        }

        Ok(Self {
            data,
            certified_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_kit::mock_principals;

    #[test]
    fn lookup() {
        let mut table = BucketLookupTable::default();
        table.insert(0, mock_principals::bob());
        table.insert(500, mock_principals::alice());
        table.insert(750, mock_principals::john());

        assert_eq!(table.get_bucket_for(0), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(50), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(150), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(499), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(500), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(600), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(749), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(750), &mock_principals::john());
        assert_eq!(table.get_bucket_for(751), &mock_principals::john());
        assert_eq!(table.get_bucket_for(10000), &mock_principals::john());
    }

    #[test]
    #[should_panic]
    fn lookup_before() {
        let mut table = BucketLookupTable::default();
        table.insert(100, mock_principals::bob());
        table.insert(500, mock_principals::alice());
        table.insert(750, mock_principals::john());

        table.get_bucket_for(10);
    }

    #[test]
    #[should_panic]
    fn lookup_empty() {
        let table = BucketLookupTable::default();
        table.get_bucket_for(0);
    }

    #[test]
    #[should_panic]
    fn invalid_start_position() {
        let mut table = BucketLookupTable::default();
        table.insert(100, mock_principals::bob());
        table.insert(50, mock_principals::alice());
    }

    #[test]
    fn pop() {
        let mut table = BucketLookupTable::default();
        table.insert(0, mock_principals::bob());
        table.insert(500, mock_principals::alice());
        table.insert(750, mock_principals::john());

        assert_eq!(table.pop(), Some((750, mock_principals::john())));

        let id = TransactionIdKey::from(750);
        assert_eq!(table.certified_map.get(id.as_ref()), None);

        table.insert(600, mock_principals::xtc());
        assert_eq!(table.get_bucket_for(599), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(600), &mock_principals::xtc());
    }

    #[test]
    fn witness() {
        let mut table = BucketLookupTable::default();

        table.insert(0, mock_principals::bob());
        let hash_0 = table.root_hash();
        assert_eq!(table.gen_witness(0).reconstruct(), hash_0);
        assert_eq!(table.gen_witness(10).reconstruct(), hash_0);

        table.insert(500, mock_principals::alice());
        let hash_500 = table.root_hash();
        assert_ne!(hash_0, hash_500, "Hash of the table should change.");

        assert_eq!(table.gen_witness(0).reconstruct(), hash_500);
        assert_eq!(table.gen_witness(10).reconstruct(), hash_500);
        assert_eq!(table.gen_witness(499).reconstruct(), hash_500);
        assert_eq!(table.gen_witness(500).reconstruct(), hash_500);
        assert_eq!(table.gen_witness(501).reconstruct(), hash_500);

        // The same table should have the same hash.
        table.pop();
        assert_eq!(hash_0, table.root_hash());
        table.insert(500, mock_principals::alice());

        table.insert(750, mock_principals::john());
        let hash_750 = table.root_hash();
        assert_ne!(hash_0, hash_750, "Hash of the table should change.");
        assert_ne!(hash_500, hash_750, "Hash of the table should change.");

        assert_eq!(table.gen_witness(0).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(10).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(499).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(500).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(501).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(749).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(750).reconstruct(), hash_750);
        assert_eq!(table.gen_witness(751).reconstruct(), hash_750);
    }

    #[test]
    fn serde() {
        let mut table = BucketLookupTable::default();
        table.insert(0, mock_principals::bob());
        table.insert(500, mock_principals::alice());
        table.insert(750, mock_principals::john());

        let expected_hashtree = table.gen_witness(730);

        let encoded = serde_cbor::to_vec(&table).expect("Failed to serialize.");
        let table =
            serde_cbor::from_slice::<BucketLookupTable>(&encoded).expect("Failed to deserialize.");

        assert_eq!(table.get_bucket_for(0), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(50), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(150), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(499), &mock_principals::bob());
        assert_eq!(table.get_bucket_for(500), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(600), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(749), &mock_principals::alice());
        assert_eq!(table.get_bucket_for(750), &mock_principals::john());
        assert_eq!(table.get_bucket_for(751), &mock_principals::john());
        assert_eq!(table.get_bucket_for(10000), &mock_principals::john());

        let actual_hashtree = table.gen_witness(730);
        assert_eq!(
            format!("{:?}", actual_hashtree),
            format!("{:?}", expected_hashtree)
        );
    }
}
