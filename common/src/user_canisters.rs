use crate::canister_list::CanisterList;
use crate::{RootBucketId, UserId};
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

#[derive(Default)]
pub struct UserCanisters {
    data: RbTree<UserId, CanisterList>,
    len: usize,
}

impl UserCanisters {
    /// Insert the RootBucketId of a token contract to a user's list.
    pub fn insert(&mut self, user: UserId, canister: RootBucketId) {
        let mut modified = false;
        self.data.modify(user.as_ref(), |list| {
            list.push(canister);
            modified = true;
        });
        if !modified {
            let mut list = CanisterList::new();
            list.push(canister);
            self.data.insert(user, list);
            self.len += 1;
        }
    }

    /// Return the list of canisters associated with a user.
    pub fn get(&self, user: &UserId) -> &[RootBucketId] {
        self.data
            .get(user.as_ref())
            .map(|l| l.as_slice())
            .unwrap_or_default()
    }

    /// Generate the HashTree witness for the `get` call.
    pub fn witness(&self, user: &UserId) -> HashTree {
        self.data.witness(user.as_ref())
    }
}

impl AsHashTree for UserCanisters {
    #[inline(always)]
    fn root_hash(&self) -> Hash {
        self.data.root_hash()
    }

    #[inline(always)]
    fn as_hash_tree(&self) -> HashTree<'_> {
        self.data.as_hash_tree()
    }
}

impl Serialize for UserCanisters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(Some(self.len))?;

        self.data.for_each(|key, value| {
            s.serialize_entry(key, value)
                .expect("Serialization failed.");
        });

        s.end()
    }
}
