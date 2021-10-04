use crate::index::Index;
use crate::transaction::Event;
use ic_certified_map::HashTree::Pruned;
use ic_certified_map::{fork, fork_hash, leaf_hash, AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
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
    event_hashes: RbTree<EventKey, Hash>,
    /// The offset of this bucket, i.e the actual id of the first event in the bucket.
    global_offset: u64,
    /// Same as `global_offset` but is the encoded big endian, this struct should own this data
    /// since it is used in the HashTree, so whenever we want to pass a reference to a BE encoded
    /// value of the `global_offset` we can use this slice.
    global_offset_be: [u8; 8],
    /// Maps each user principal id to the vector of events they have.
    user_indexer: Index,
    /// Maps each token contract principal id to the vector of events inserted by that token.
    contract_indexer: Index,
    /// All of the events in this bucket, we store a pointer to an allocated memory. Which is used
    /// only internally in this struct. And this Vec should be considered the actual owner of this
    /// pointers.
    /// So this should be the last thing that will be dropped.
    events: Vec<NonNull<Event>>,
}

pub struct EventKey([u8; 4]);

impl From<u32> for EventKey {
    #[inline(always)]
    fn from(n: u32) -> Self {
        EventKey(n.to_be_bytes())
    }
}

impl AsRef<[u8]> for EventKey {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Bucket {
    /// Create a new bucket with the given global offset.
    #[inline]
    pub fn new(offset: u64) -> Self {
        Bucket {
            events: vec![],
            event_hashes: RbTree::new(),
            global_offset: offset,
            global_offset_be: offset.to_be_bytes(),
            user_indexer: Index::default(),
            contract_indexer: Index::default(),
        }
    }

    /// Try to insert an event into the bucket.
    pub fn insert(&mut self, contract: &Principal, event: Event) -> u64 {
        let local_index = self.events.len() as u32;
        let hash = event.hash();
        let event: NonNull<Event> = Box::leak(Box::new(event)).into();
        let eve = unsafe { event.as_ref() };

        // Update the indexers for the transaction.
        self.contract_indexer.insert(contract, event, &hash);
        for user in eve.extract_principal_ids() {
            self.user_indexer.insert(user, event, &hash);
        }

        // Insert the event itself.
        self.event_hashes.insert(local_index.into(), hash);
        self.events.push(event);

        self.global_offset + (local_index as u64)
    }

    /// Create the hash of the left virtual node.
    #[inline]
    fn left_v_hash(&self) -> Hash {
        let offset_hash = leaf_hash(&self.global_offset_be);
        fork_hash(&self.event_hashes.root_hash(), &offset_hash)
    }

    /// Create the hash of the right virtual node.
    #[inline]
    fn right_v_hash(&self) -> Hash {
        fork_hash(
            &self.user_indexer.root_hash(),
            &self.contract_indexer.root_hash(),
        )
    }

    /// Return the transactions associated with a user's principal id at the given page.
    #[inline]
    pub fn get_transactions_for_user(&self, principal: &Principal, page: u32) -> Vec<&Event> {
        if let Some(data) = self.user_indexer.get(principal, page) {
            data.iter().map(|v| unsafe { v.as_ref() }).collect()
        } else {
            vec![]
        }
    }

    /// Return the last page number associated with the given user.
    #[inline]
    pub fn last_page_for_user(&self, principal: &Principal) -> u32 {
        self.user_indexer.last_page(principal)
    }

    /// Return the transactions associated with a token's principal id at the given page.
    #[inline]
    pub fn get_transactions_for_contract(&self, principal: &Principal, page: u32) -> Vec<&Event> {
        if let Some(data) = self.contract_indexer.get(principal, page) {
            data.iter().map(|v| unsafe { v.as_ref() }).collect()
        } else {
            vec![]
        }
    }

    /// Return the last page number associated with the given token.
    #[inline]
    pub fn last_page_for_contract(&self, principal: &Principal) -> u32 {
        self.contract_indexer.last_page(principal)
    }

    /// Return the witness that can be used to prove the response from get_transactions_for_user.
    #[inline]
    pub fn witness_transactions_for_user(&self, principal: &Principal, page: u32) -> HashTree {
        fork(
            Pruned(self.left_v_hash()),
            fork(
                self.user_indexer.witness(principal, page),
                Pruned(self.contract_indexer.root_hash()),
            ),
        )
    }

    /// Return the witness that can be used to prove the response from get_transactions_for_token.
    #[inline]
    pub fn witness_transactions_for_contract(&self, principal: &Principal, page: u32) -> HashTree {
        fork(
            Pruned(self.left_v_hash()),
            fork(
                Pruned(self.user_indexer.root_hash()),
                self.contract_indexer.witness(principal, page),
            ),
        )
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
        if id < self.global_offset {
            fork(
                fork(
                    Pruned(self.event_hashes.root_hash()),
                    HashTree::Leaf(&self.global_offset_be),
                ),
                Pruned(self.right_v_hash()),
            )
        } else {
            let local = (id - self.global_offset) as u32;
            fork(
                fork(
                    self.event_hashes.witness(&local.to_be_bytes()),
                    HashTree::Leaf(&self.global_offset_be),
                ),
                Pruned(self.right_v_hash()),
            )
        }
    }
}

impl AsHashTree for Bucket {
    fn root_hash(&self) -> Hash {
        fork_hash(&self.left_v_hash(), &self.right_v_hash())
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        fork(
            fork(
                self.event_hashes.as_hash_tree(),
                HashTree::Leaf(&self.global_offset_be),
            ),
            fork(
                self.user_indexer.as_hash_tree(),
                self.contract_indexer.as_hash_tree(),
            ),
        )
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
        let mut s = serializer.serialize_seq(Some(self.events.len()))?;
        for ev in &self.events {
            s.serialize_element(unsafe { ev.as_ref() })?;
        }
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Operation;
    use ic_kit::mock_principals;

    fn e(memo: u32, caller: Principal) -> Event {
        Event {
            time: 0,
            caller,
            amount: 0,
            fee: 0,
            memo,
            from: None,
            to: caller,
            operation: Operation::Transfer,
        }
    }

    /// root_hash and as_hash_tree should use the same tree layout.
    #[test]
    fn test_hash_tree() {
        let mut bucket = Bucket::new(0);
        bucket.insert(&mock_principals::xtc(), e(0, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(1, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(2, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(3, mock_principals::alice()));
        assert_eq!(bucket.as_hash_tree().reconstruct(), bucket.root_hash());
    }

    /// This test tires to see if the witness created for a lookup is minimal
    /// and reconstructs to the root_hash.
    #[test]
    fn test_witness_transaction() {
        let mut bucket = Bucket::new(0);
        bucket.insert(&mock_principals::xtc(), e(0, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(1, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(2, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(3, mock_principals::alice()));

        let event = bucket.get_transaction(1).unwrap();
        let witness = bucket.witness_transaction(1);
        assert_eq!(event.memo, 1);
        assert_eq!(witness.reconstruct(), bucket.root_hash());
    }

    #[test]
    fn test_witness_transaction_large() {
        let mut bucket = Bucket::new(0);
        bucket.insert(&mock_principals::xtc(), e(0, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(1, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(2, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(3, mock_principals::alice()));

        assert_eq!(bucket.get_transaction(4).is_none(), true);

        let witness = bucket.witness_transaction(4);
        assert_eq!(witness.reconstruct(), bucket.root_hash());
    }

    #[test]
    fn test_witness_transaction_below_offset() {
        let mut bucket = Bucket::new(10);
        bucket.insert(&mock_principals::xtc(), e(10, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(11, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(12, mock_principals::alice()));
        bucket.insert(&mock_principals::xtc(), e(13, mock_principals::alice()));

        assert_eq!(bucket.get_transaction(5).is_none(), true);
        let witness = bucket.witness_transaction(5);
        assert_eq!(witness.reconstruct(), bucket.root_hash());
    }

    #[test]
    fn test_witness_user_transactions() {
        let mut bucket = Bucket::new(0);

        for i in 0..5000 {
            if i % 27 == 0 {
                bucket.insert(&mock_principals::xtc(), e(i, mock_principals::bob()));
            } else {
                bucket.insert(&mock_principals::xtc(), e(i, mock_principals::alice()));
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
    fn test_witness_token_transactions() {
        let mut bucket = Bucket::new(0);

        for i in 0..2500 {
            if i % 13 == 0 {
                bucket.insert(&mock_principals::bob(), e(i, mock_principals::xtc()));
            } else {
                bucket.insert(&mock_principals::xtc(), e(i, mock_principals::alice()));
            }
        }

        let mut count = 0;

        for page in 0.. {
            let principal = mock_principals::bob();
            let data = bucket.get_transactions_for_contract(&principal, page);
            let witness = bucket.witness_transactions_for_contract(&principal, page);
            let len = data.len();

            assert_eq!(witness.reconstruct(), bucket.root_hash());

            count += len;

            if len == 0 {
                break;
            }
        }

        // floor(2500 / 13) + 1 = 193
        assert_eq!(count, 193);
    }
}
