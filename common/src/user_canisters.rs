use crate::canister_list::CanisterList;
use crate::{RootBucketId, UserId};
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

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

impl<'de> Deserialize<'de> for UserCanisters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(UserCanistersVisitor)
    }
}

struct UserCanistersVisitor;

impl<'de> Visitor<'de> for UserCanistersVisitor {
    type Value = UserCanisters;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "expected a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut data = RbTree::default();
        let mut len = 0;

        loop {
            if let Some((key, value)) = map.next_entry::<Vec<u8>, CanisterList>()? {
                let principal = Principal::from_slice(&key);
                data.insert(principal, value);
                len += 1;
                continue;
            }

            break;
        }

        Ok(UserCanisters { data, len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_kit::mock_principals;

    #[test]
    fn serde() {
        let mut data = UserCanisters::default();
        data.insert(mock_principals::alice(), mock_principals::xtc());
        data.insert(mock_principals::alice(), mock_principals::bob());
        data.insert(mock_principals::john(), mock_principals::alice());
        data.insert(mock_principals::john(), mock_principals::xtc());
        data.insert(mock_principals::john(), mock_principals::bob());

        let serialized = serde_cbor::to_vec(&data).expect("Failed to serialize");
        let actual =
            serde_cbor::from_slice::<UserCanisters>(&serialized).expect("Failed to deserialize");

        assert_eq!(
            actual.get(&mock_principals::alice()),
            data.get(&mock_principals::alice())
        );
        assert_eq!(
            actual.get(&mock_principals::john()),
            data.get(&mock_principals::john())
        );
    }
}
