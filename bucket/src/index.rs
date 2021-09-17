use crate::transaction::Event;
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ptr::NonNull;

/// How many Transaction IDs per page.
pub const PAGE_CAPACITY: usize = 64;

/// Type used for representing the page number.
type PageNumber = u32;

#[derive(Default)]
pub struct Index {
    pager: BTreeMap<Principal, PageNumber>,
    data: RbTree<IndexKey, IndexPage>,
}

/// The key in the indexer which points to a page for a principal id.
/// structure:
/// u8      Principal length
/// u8;29   Principal inner
/// u8;4    Page number, u32 as Big Endian
struct IndexKey([u8; 34]);

#[derive(Default)]
struct IndexPage {
    data: Vec<NonNull<Event>>,
    hash: Hash,
}

impl Index {
    /// Insert a new transaction into the lookup table of the given principal id.
    /// The second parameter should be the hash of the passed event.
    pub fn insert(&mut self, principal: &Principal, event: NonNull<Event>, hash: &Hash) {
        let mut inserted = false;

        let next_page = if let Some(&page_no) = self.pager.get(principal) {
            let key = IndexKey::new(principal, page_no);

            self.data.modify(key.as_ref(), |page| {
                inserted = page.insert(event, hash);
            });

            page_no + 1
        } else {
            0
        };

        // Create a new page.
        if !inserted {
            let mut page = IndexPage::default();
            page.insert(event, hash);

            let key = IndexKey::new(principal, next_page);
            self.data.insert(key, page);
            self.pager.insert(principal.clone(), next_page);
        }
    }

    /// Create a witness proving the data returned by get.
    #[inline]
    pub fn witness(&self, principal: &Principal, page: u32) -> HashTree {
        let key = IndexKey::new(principal, page);
        self.data.witness(key.as_ref())
    }

    /// Return the data associated with the given page.
    #[inline]
    pub fn get(&self, principal: &Principal, page: u32) -> Option<&Vec<NonNull<Event>>> {
        let key = IndexKey::new(principal, page);
        if let Some(page) = self.data.get(key.as_ref()) {
            Some(&page.data)
        } else {
            None
        }
    }
}

impl AsHashTree for Index {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        self.data.root_hash()
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree<'_> {
        self.data.as_hash_tree()
    }
}

impl IndexKey {
    /// Construct a new index-key from a principal id and a page number.
    #[inline(always)]
    pub fn new(principal: &Principal, page: PageNumber) -> Self {
        let mut buffer = [0u8; 34];
        let principal_slice = principal.as_slice();
        let page_slice = page.to_be_bytes();

        buffer[0] = principal_slice.len() as u8;
        for i in 0..principal_slice.len() {
            buffer[i + 1] = principal_slice[i];
        }

        for i in 0..4 {
            buffer[i + 30] = page_slice[i];
        }

        IndexKey(buffer)
    }
}

impl AsRef<[u8]> for IndexKey {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl IndexPage {
    /// Try to insert a local transaction id into the page, returns the success status.
    #[inline]
    pub fn insert(&mut self, event: NonNull<Event>, hash: &Hash) -> bool {
        if self.data.len() == PAGE_CAPACITY {
            return false;
        }

        self.data.push(event);

        // Compute the new hash.
        let mut h = Sha256::new();
        h.update(&self.hash);
        h.update(hash);
        self.hash = h.finalize().into();

        true
    }
}

impl AsHashTree for IndexPage {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        self.hash.root_hash()
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree<'_> {
        self.hash.as_hash_tree()
    }
}
