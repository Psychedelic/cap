//! Extends the ic-certified-map to add witness for multiple keys.
use crate::bucket::EventKey;
use crate::index::IndexPageBeIterator;
use ic_certified_map::{
    fork, fork_hash, labeled, labeled_hash, AsHashTree, Hash,
    HashTree::{self, Empty, Pruned},
    Node, RbTree,
};
use std::cmp::Ordering;
use std::iter::Peekable;

enum Color {
    _Red,
    _Black,
}

struct NodeVisible<K, V> {
    key: K,
    value: V,
    left: *mut NodeVisible<K, V>,
    right: *mut NodeVisible<K, V>,
    _color: Color,
    subtree_hash: Hash,
}

impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> NodeVisible<K, V> {
    unsafe fn data_hash(n: *mut Self) -> Hash {
        debug_assert!(!n.is_null());
        labeled_hash((*n).key.as_ref(), &(*n).value.root_hash())
    }

    unsafe fn left_hash_tree<'a>(n: *mut Self) -> HashTree<'a> {
        debug_assert!(!n.is_null());
        if (*n).left.is_null() {
            Empty
        } else {
            Pruned((*(*n).left).subtree_hash)
        }
    }

    unsafe fn subtree_with<'a>(
        n: *mut Self,
        f: impl FnOnce(&'a V) -> HashTree<'a>,
    ) -> HashTree<'a> {
        debug_assert!(!n.is_null());
        labeled((*n).key.as_ref(), f(&(*n).value))
    }
}

struct RbTreeVisible<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> {
    root: *mut NodeVisible<K, V>,
}

impl<K, V> From<&Node<K, V>> for &NodeVisible<K, V> {
    #[inline(always)]
    fn from(n: &Node<K, V>) -> Self {
        unsafe { std::mem::transmute(n) }
    }
}
impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> From<&RbTree<K, V>>
    for &RbTreeVisible<K, V>
{
    #[inline(always)]
    fn from(t: &RbTree<K, V>) -> Self {
        unsafe { std::mem::transmute(t) }
    }
}

/// Build the witness tree for the events which will include all of the given keys.
pub fn build_events_witness<'a>(
    tree: &'a RbTree<EventKey, Hash>,
    keys: IndexPageBeIterator<'a>,
) -> HashTree<'a> {
    println!("Called");

    unsafe fn build<'a, 'p>(
        keys: &'p mut Peekable<IndexPageBeIterator<'a>>,
        n: *mut NodeVisible<EventKey, Hash>,
    ) -> HashTree<'a> {
        if n.is_null() {
            return Empty;
        }

        // If we're not looking for anymore keys, just prune this entire subtree.
        // Because of the IndexPageBeIterator order, this key is the min of all the keys we're
        // looking for.
        // so key < every other key.
        let key = match keys.peek() {
            Some(key) => *key,
            None => return Pruned(NodeVisible::data_hash(n)),
        };

        let mut s = [0; 4];
        s.copy_from_slice(key);

        match key.cmp((*n).key.as_ref()) {
            Ordering::Equal => {
                // Consume the key.
                keys.next();
                // There might be more keys to lookup that are on the right node.
                let subtree = build(keys, (*n).right);
                three_way_fork(
                    NodeVisible::left_hash_tree(n),
                    NodeVisible::subtree_with(n, |n| n.as_hash_tree()),
                    subtree,
                )
            }
            Ordering::Less => {
                // Build the sub-tree for the current key we're looking for.
                let left_subtree = build(keys, (*n).left);

                // Now the new current key might be equal to this node. In that case we want to
                // include and not prune it.
                let prune_current = match keys.peek() {
                    Some(&x) if x == (*n).key.as_ref() => false,
                    _ => true,
                };

                let mid_subtree = if prune_current {
                    Pruned(NodeVisible::data_hash(n))
                } else {
                    // If we're going to include this key, we should not look for it any further.
                    // So it's consumed.
                    keys.next();
                    NodeVisible::subtree_with(n, |n| n.as_hash_tree())
                };

                let right_subtree = build(keys, (*n).right);

                three_way_fork(left_subtree, mid_subtree, right_subtree)
            }
            Ordering::Greater => {
                let subtree = build(keys, (*n).right);
                three_way_fork(
                    NodeVisible::left_hash_tree(n),
                    Pruned(NodeVisible::data_hash(n)),
                    subtree,
                )
            }
        }
    }

    let mut keys = keys.peekable();
    let tree: &RbTreeVisible<EventKey, Hash> = tree.into();

    unsafe { build(&mut keys, tree.root) }
}

fn three_way_fork<'a>(l: HashTree<'a>, m: HashTree<'a>, r: HashTree<'a>) -> HashTree<'a> {
    match (l, m, r) {
        (Empty, m, Empty) => m,
        (l, m, Empty) => fork(l, m),
        (Empty, m, r) => fork(m, r),
        (Pruned(l_hash), Pruned(m_hash), Pruned(r_hash)) => {
            Pruned(fork_hash(&l_hash, &fork_hash(&m_hash, &r_hash)))
        }
        (l, Pruned(m_hash), Pruned(r_hash)) => fork(l, Pruned(fork_hash(&m_hash, &r_hash))),
        (l, m, r) => fork(l, fork(m, r)),
    }
}

// TODO(qti3e) Test!
