use crate::transaction::Event;
use certified_vars::Paged;
use certified_vars::{rbtree::RbTree, AsHashTree, Hash, HashTree};
use ic_kit::Principal;
use serde::ser::{SerializeSeq, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::alloc::{dealloc, Layout};
use std::ptr;
use std::ptr::NonNull;

/// A bucket contains a series of transactions and appropriate indexers.
///
/// This structure exposes a virtual merkle-tree in the following form:
///
/// 0: event_hashes
/// 1: offset
/// 3: user_indexer
/// 4: token_indexer
///
/// ```text
///       ROOT
///      /    \
///     /      \
///    V        V
///   /  \     /  \
///  0    1   3    4
/// ```
pub struct Bucket {
    /// Map each local Transaction ID to its hash.
    event_hashes: RbTree<u32, Hash>,
    /// ID of the current contract.
    contract: Principal,
    /// The offset of this bucket, i.e the actual id of the first event in the bucket.
    global_offset: u64,
    /// Same as `global_offset` but is the encoded big endian, this struct should own this data
    /// since it is used in the HashTree, so whenever we want to pass a reference to a BE encoded
    /// value of the `global_offset` we can use this slice.
    global_offset_be: [u8; 8],
    /// Maps each user principal id to the vector of events they have.
    user_indexer: Paged<Principal, NonNull<Event>, 64>,
    /// Maps contract id to each transaction page.
    contract_indexer: Paged<Principal, NonNull<Event>, 64>,
    /// All of the events in this bucket, we store a pointer to an allocated memory. Which is used
    /// only internally in this struct. And this Vec should be considered the actual owner of this
    /// pointers.
    /// So this should be the last thing that will be dropped.
    events: Vec<NonNull<Event>>,
}

impl Bucket {
    /// Create a new bucket with the given global offset.
    #[inline]
    pub fn new(contract: Principal, offset: u64) -> Self {
        Bucket {
            events: vec![],
            contract,
            event_hashes: RbTree::new(),
            global_offset: offset,
            global_offset_be: offset.to_be_bytes(),
            user_indexer: Paged::new(),
            contract_indexer: Paged::new(),
        }
    }

    /// Return the total number of transactions.
    #[inline]
    pub fn size(&self) -> u64 {
        self.global_offset + (self.events.len() as u64)
    }

    /// Return the total number of items in this bucket.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `tru` if there are no events in this bucket.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Try to insert an event into the bucket.
    pub fn insert(&mut self, event: Event) -> u64 {
        let local_index = self.events.len() as u32;
        let hash = event.hash();
        let event: NonNull<Event> = Box::leak(Box::new(event)).into();
        let eve = unsafe { event.as_ref() };

        // Update the indexers for the transaction.
        self.contract_indexer.insert(self.contract, event);
        for user in eve.extract_principal_ids() {
            self.user_indexer.insert(*user, event);
        }

        // Insert the event itself.
        self.event_hashes.insert(local_index, hash);
        self.events.push(event);

        self.global_offset + (local_index as u64)
    }

    /// Return the transactions associated with a user's principal id at the given page.
    #[inline]
    pub fn get_transactions_for_user(&self, principal: &Principal, page: u32) -> Vec<&Event> {
        if let Some(data) = self.user_indexer.get(principal, page as usize) {
            data.iter().map(|v| unsafe { v.as_ref() }).collect()
        } else {
            vec![]
        }
    }

    /// Return the last page number associated with the given user.
    #[inline]
    pub fn last_page_for_user(&self, principal: &Principal) -> u32 {
        self.user_indexer
            .get_last_page_number(principal)
            .unwrap_or(0) as u32
    }

    /// Return the transactions associated with a token's principal id at the given page.
    #[inline]
    pub fn get_transactions_for_contract(&self, principal: &Principal, page: u32) -> Vec<&Event> {
        if let Some(data) = self.contract_indexer.get(principal, page as usize) {
            data.iter().map(|v| unsafe { v.as_ref() }).collect()
        } else {
            vec![]
        }
    }

    /// Return the last page number associated with the given token contract.
    #[inline]
    pub fn last_page_for_contract(&self, principal: &Principal) -> u32 {
        self.contract_indexer
            .get_last_page_number(principal)
            .unwrap_or(0) as u32
    }

    /// Return the witness that can be used to prove the response from get_transactions_for_user.
    #[inline]
    pub fn witness_transactions_for_user(&self, principal: &Principal, page: u32) -> HashTree {
        todo!()
    }

    /// Return the witness that can be used to prove the response from get_transactions_for_token.
    #[inline]
    pub fn witness_transactions_for_contract(&self, principal: &Principal, page: u32) -> HashTree {
        todo!()
    }

    /// Return a transaction by its global id.
    #[inline]
    pub fn get_transaction(&self, id: u64) -> Option<&Event> {
        if id < self.global_offset {
            None
        } else {
            let local = (id - self.global_offset) as usize;
            if local < self.events.len() {
                Some(unsafe { self.events[local].as_ref() })
            } else {
                None
            }
        }
    }

    /// Return a witness which proves the response returned by get_transaction.
    #[inline]
    pub fn witness_transaction(&self, id: u64) -> HashTree {
        todo!()
    }
}

impl AsHashTree for Bucket {
    fn root_hash(&self) -> Hash {
        todo!()
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        todo!()
    }
}

impl Drop for Bucket {
    fn drop(&mut self) {
        unsafe {
            for event in &self.events {
                let as_mut_ref = &mut (*event.as_ptr());
                ptr::drop_in_place(as_mut_ref);
                dealloc(event.cast().as_ptr(), Layout::for_value(event.as_ref()));
            }
        }
    }
}

impl Serialize for Bucket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Events<'a>(&'a Vec<NonNull<Event>>);
        impl<'a> Serialize for Events<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut s = serializer.serialize_seq(Some(self.0.len()))?;
                for ev in self.0 {
                    s.serialize_element(unsafe { ev.as_ref() })?;
                }
                s.end()
            }
        }

        let mut s = serializer.serialize_tuple(3)?;
        s.serialize_element(&self.global_offset)?;
        s.serialize_element(&self.contract)?;
        s.serialize_element(&Events(&self.events))?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Bucket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BucketDe(u64, Principal, Vec<Event>);

        let data = BucketDe::deserialize(deserializer)?;
        let mut bucket = Bucket::new(data.1, data.0);

        for event in data.2 {
            bucket.insert(event);
        }

        Ok(bucket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_kit::mock_principals;

    fn e(time: u64, caller: Principal) -> Event {
        Event {
            time,
            caller,
            operation: "transfer".into(),
            details: vec![],
        }
    }

    /// root_hash and as_hash_tree should use the same tree layout.
    #[test]
    fn test_hash_tree() {
        let mut bucket = Bucket::new(mock_principals::xtc(), 0);
        bucket.insert(e(0, mock_principals::alice()));
        bucket.insert(e(1, mock_principals::alice()));
        bucket.insert(e(2, mock_principals::alice()));
        bucket.insert(e(3, mock_principals::alice()));
        assert_eq!(bucket.as_hash_tree().reconstruct(), bucket.root_hash());
    }

    /// This test tires to see if the witness created for a lookup is minimal
    /// and reconstructs to the root_hash.
    #[test]
    fn test_witness_transaction() {
        let mut bucket = Bucket::new(mock_principals::xtc(), 0);
        bucket.insert(e(0, mock_principals::alice()));
        bucket.insert(e(1, mock_principals::alice()));
        bucket.insert(e(2, mock_principals::alice()));
        bucket.insert(e(3, mock_principals::alice()));

        let event = bucket.get_transaction(1).unwrap();
        let witness = bucket.witness_transaction(1);
        assert_eq!(event.time, 1);
        assert_eq!(witness.reconstruct(), bucket.root_hash());
    }

    #[test]
    fn test_witness_transaction_large() {
        let mut bucket = Bucket::new(mock_principals::xtc(), 0);
        bucket.insert(e(0, mock_principals::alice()));
        bucket.insert(e(1, mock_principals::alice()));
        bucket.insert(e(2, mock_principals::alice()));
        bucket.insert(e(3, mock_principals::alice()));

        assert_eq!(bucket.get_transaction(4).is_none(), true);

        let witness = bucket.witness_transaction(4);
        assert_eq!(witness.reconstruct(), bucket.root_hash());
    }

    #[test]
    fn test_witness_transaction_below_offset() {
        let mut bucket = Bucket::new(mock_principals::xtc(), 10);
        bucket.insert(e(10, mock_principals::alice()));
        bucket.insert(e(11, mock_principals::alice()));
        bucket.insert(e(12, mock_principals::alice()));
        bucket.insert(e(13, mock_principals::alice()));

        assert_eq!(bucket.get_transaction(5).is_none(), true);
        let witness = bucket.witness_transaction(5);
        assert_eq!(witness.reconstruct(), bucket.root_hash());
    }

    #[test]
    fn test_witness_user_transactions() {
        let mut bucket = Bucket::new(mock_principals::xtc(), 0);

        for i in 0..5000 {
            if i % 27 == 0 {
                bucket.insert(e(i, mock_principals::bob()));
            } else {
                bucket.insert(e(i, mock_principals::alice()));
            }
        }

        let mut count = 0;

        for page in 0.. {
            let principal = mock_principals::bob();
            let data = bucket.get_transactions_for_user(&principal, page);
            let witness = bucket.witness_transactions_for_user(&principal, page);
            let len = data.len();

            assert_eq!(witness.reconstruct(), bucket.root_hash());

            count += len;

            if len == 0 {
                break;
            }
        }

        // floor(5000 / 27) + 1 = 186
        assert_eq!(count, 186);
    }

    #[test]
    fn serde() {
        let mut bucket = Bucket::new(mock_principals::xtc(), 0);
        bucket.insert(e(0, mock_principals::alice()));
        bucket.insert(e(1, mock_principals::alice()));
        bucket.insert(e(2, mock_principals::alice()));
        bucket.insert(e(3, mock_principals::alice()));
        let expected = bucket.root_hash();

        let data: Vec<u8> = serde_cbor::to_vec(&bucket).unwrap();
        let bucket: Bucket = serde_cbor::from_slice(&data).unwrap();
        assert_eq!(bucket.root_hash(), expected);
    }
}
