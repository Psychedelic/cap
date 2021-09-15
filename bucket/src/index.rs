use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use std::collections::BTreeMap;

/// How many Transaction IDs per page.
pub const PAGE_CAPACITY: usize = 64;
/// Number of bytes required to store each page's data. 256 bytes.
const PAGE_CAPACITY_BYTES: usize = PAGE_CAPACITY * std::mem::size_of::<u32>();

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
    /// An array of u32s encoded as big endian.
    data: Vec<u8>,
}

impl Index {
    /// Insert a new local transaction id into the lookup table of the given principal id.
    pub fn insert(&mut self, principal: &Principal, id: u32) {
        let mut inserted = false;

        let next_page = if let Some(&page_no) = self.pager.get(principal) {
            let key = IndexKey::new(principal, page_no);

            self.data.modify(key.as_ref(), |page| {
                inserted = page.insert(id);
            });

            page_no + 1
        } else {
            0
        };

        // Create a new page.
        if !inserted {
            let mut page = IndexPage::default();
            page.insert(id);

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

    /// Get a page from the indexer.
    #[inline]
    pub fn get(&self, principal: &Principal, page: u32) -> Option<IndexPageIterator> {
        let key = IndexKey::new(principal, page);
        self.data.get(key.as_ref()).map(IndexPage::iter)
    }

    /// Get a BE iterator over the transaction ids in a page.
    #[inline]
    pub fn get_be(&self, principal: &Principal, page: u32) -> Option<IndexPageBeIterator> {
        let key = IndexKey::new(principal, page);
        self.data.get(key.as_ref()).map(IndexPage::iter_be)
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

    #[inline]
    pub fn parse(&self) -> (Principal, PageNumber) {
        let principal_len = self.0[0] as usize;
        let principal_slice = &self.0[1..][..principal_len];
        let mut page_slice = [0u8; 4];
        page_slice.copy_from_slice(&self.0[30..]);
        (
            Principal::from_slice(&principal_slice),
            u32::from_be_bytes(page_slice),
        )
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
    pub fn insert(&mut self, id: u32) -> bool {
        if self.data.len() == PAGE_CAPACITY_BYTES {
            return false;
        }

        let slice = id.to_be_bytes();
        self.data.extend_from_slice(&slice);

        if self.data.capacity() > PAGE_CAPACITY_BYTES {
            let vec = Vec::with_capacity(PAGE_CAPACITY_BYTES);
            let data = std::mem::replace(&mut self.data, vec);
            for b in data.into_iter() {
                self.data.push(b);
            }
        }

        true
    }

    /// Return the number of items inserted into this page.
    #[inline]
    pub fn len(&self) -> usize {
        // div by 4.
        self.data.len() >> 2
    }

    /// Return the transaction id at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<u32> {
        let offset = index * 4;
        if offset >= self.data.len() {
            return None;
        }
        let mut buffer = [0u8; 4];
        buffer.copy_from_slice(&self.data[offset..][..4]);
        Some(u32::from_be_bytes(buffer))
    }

    /// Create an iterator over the transaction ids in this page.
    #[inline]
    pub fn iter(&self) -> IndexPageIterator {
        IndexPageIterator {
            cursor: 0,
            page: &self,
        }
    }

    /// Create an iterator over the transaction ids as big endian in this page.
    #[inline]
    pub fn iter_be(&self) -> IndexPageBeIterator {
        IndexPageBeIterator {
            cursor: 0,
            page: &self,
        }
    }
}

impl AsHashTree for IndexPage {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        self.data.root_hash()
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree<'_> {
        self.data.as_hash_tree()
    }
}

pub struct IndexPageIterator<'a> {
    cursor: usize,
    page: &'a IndexPage,
}

impl<'a> Iterator for IndexPageIterator<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.cursor;
        self.cursor += 1;
        self.page.get(index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.page.len() - self.cursor;
        (rem, Some(rem))
    }

    fn count(self) -> usize {
        self.page.len() - self.cursor
    }
}

pub struct IndexPageBeIterator<'a> {
    cursor: usize,
    page: &'a IndexPage,
}

impl<'a> Iterator for IndexPageBeIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.cursor << 2; // mul 4
        self.cursor += 1;
        if offset >= self.page.data.len() {
            None
        } else {
            Some(&self.page.data[offset..offset + 4])
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.page.len() - self.cursor;
        (rem, Some(rem))
    }

    fn count(self) -> usize {
        self.page.len() - self.cursor
    }
}
