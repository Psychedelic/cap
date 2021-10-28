use ic_certified_map::HashTree::Leaf;
use ic_certified_map::{leaf_hash, AsHashTree, Hash, HashTree, RbTree};
use ic_kit::Principal;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

/// A data structure that maps a canister id to another canister id and
#[derive(Default)]
pub struct CanisterMap {
    data: RbTree<Principal, PrincipalBytes>,
}

struct PrincipalBytes(Principal);

impl From<Principal> for PrincipalBytes {
    #[inline]
    fn from(p: Principal) -> Self {
        Self(p)
    }
}

impl AsHashTree for PrincipalBytes {
    #[inline]
    fn root_hash(&self) -> Hash {
        leaf_hash(self.0.as_ref())
    }

    #[inline]
    fn as_hash_tree(&self) -> HashTree<'_> {
        Leaf(self.0.as_ref())
    }
}

impl CanisterMap {
    /// Insert the given relation into the map.
    #[inline]
    pub fn insert(&mut self, key: Principal, value: Principal) {
        self.data.insert(key, value.into());
    }

    /// Return the principal id associated with the given principal id.
    #[inline]
    pub fn get(&self, key: &Principal) -> Option<&Principal> {
        match self.data.get(key.as_ref()) {
            Some(bytes) => Some(&bytes.0),
            None => None,
        }
    }

    /// Create a HashTree witness for the value associated with the given key.
    #[inline]
    pub fn gen_witness(&self, key: &Principal) -> HashTree {
        self.data.witness(key.as_ref())
    }
}

impl AsHashTree for CanisterMap {
    fn root_hash(&self) -> Hash {
        self.data.root_hash()
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        self.data.as_hash_tree()
    }
}

impl Serialize for CanisterMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(None)?;

        self.data.for_each(|key, value| {
            s.serialize_entry(key, value.0.as_ref())
                .expect("Serialization failed.");
        });

        s.end()
    }
}

impl<'de> Deserialize<'de> for CanisterMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(CanisterMapVisitor)
    }
}

struct CanisterMapVisitor;

impl<'de> Visitor<'de> for CanisterMapVisitor {
    type Value = CanisterMap;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a map of principal id to principal id")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut data = CanisterMap::default();
        loop {
            if let Some((key, value)) = map.next_entry::<Principal, Principal>()? {
                data.insert(key, value);
            } else {
                break;
            }
        }
        Ok(data)
    }
}
