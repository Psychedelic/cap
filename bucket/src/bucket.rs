use crate::index::Index;
use crate::transaction::Event;
use ic_certified_map::{fork, fork_hash, leaf_hash, AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;

/// A bucket contains a series of transactions and appropriate indexers.
///
/// This structure exposes a virtual merkle-tree in the following form:
///
/// 0: event_hashes
/// 1: offset
/// 3: user_indexer
/// 4: token_indexer
///
/// ```txt
///       ROOT
///      /    \
///     /      \
///    V        V
///   /  \     /  \
///  0    1   3    4
/// ```
pub struct Bucket {
    events: Vec<Event>,
    event_hashes: RbTree<EventKey, Hash>,
    global_offset: u64,
    global_offset_be: [u8; 8],
    user_indexer: Index,
    token_indexer: Index,
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
    /// Try to insert an event into the bucket.
    pub fn insert(&mut self, event: Event) -> u64 {
        let local_index = self.events.len() as u32;

        // Update the indexers for the transaction.
        self.token_indexer.insert(&event.token, local_index);
        for user in event.extract_principal_ids() {
            self.user_indexer.insert(user, local_index);
        }

        // Insert the event itself.
        self.event_hashes.insert(local_index.into(), event.hash());
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
            &self.token_indexer.root_hash(),
        )
    }

    /// Return the transactions associated with a user's principal id at the given page.
    #[inline]
    pub fn get_transactions_for_user(&self, principal: &Principal, page: u32) -> Vec<&Event> {
        self.user_indexer
            .get(principal, page)
            .map(|iter| iter.map(|id| &self.events[id as usize]).collect())
            .unwrap_or_default()
    }

    /// Return the transactions associated with a token's principal id at the given page.
    #[inline]
    pub fn get_transactions_for_token(&self, principal: &Principal, page: u32) -> Vec<&Event> {
        self.token_indexer
            .get(principal, page)
            .map(|iter| iter.map(|id| &self.events[id as usize]).collect())
            .unwrap_or_default()
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
                self.token_indexer.as_hash_tree(),
            ),
        )
    }
}
