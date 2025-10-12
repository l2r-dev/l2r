//! ObjectId-optimized IndexSet implementation for maintaining insertion order
//! while providing efficient lookups and operations.

use crate::object_id::ObjectId;
use bevy::prelude::*;
use indexmap::IndexSet;
use l2r_core::model::generic_number::SimpleNumberHasher;

/// An [`IndexSet`] pre-configured to use [`SimpleNumberHasher`] hashing.
///
/// This type preserves insertion order while providing efficient O(1) lookups,
/// making it ideal for inventories where item order matters but we also need
/// fast containment checks and removals.
#[derive(Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct ObjectIdIndexSet(#[reflect(ignore)] IndexSet<ObjectId, SimpleNumberHasher>);

impl ObjectIdIndexSet {
    /// Creates an empty `ObjectIdIndexSet`.
    pub fn new() -> Self {
        Self(IndexSet::with_hasher(SimpleNumberHasher::default()))
    }

    /// Creates an empty `ObjectIdIndexSet` with the specified capacity.
    pub fn with_capacity(n: usize) -> Self {
        Self(IndexSet::with_capacity_and_hasher(
            n,
            SimpleNumberHasher::default(),
        ))
    }
}

impl<'a> IntoIterator for &'a ObjectIdIndexSet {
    type Item = &'a ObjectId;
    type IntoIter = indexmap::set::Iter<'a, ObjectId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for ObjectIdIndexSet {
    type Item = ObjectId;
    type IntoIter = indexmap::set::IntoIter<ObjectId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<ObjectId> for ObjectIdIndexSet {
    fn extend<T: IntoIterator<Item = ObjectId>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<'a> Extend<&'a ObjectId> for ObjectIdIndexSet {
    fn extend<T: IntoIterator<Item = &'a ObjectId>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl FromIterator<ObjectId> for ObjectIdIndexSet {
    fn from_iter<I: IntoIterator<Item = ObjectId>>(iterable: I) -> Self {
        let mut set = Self::new();
        set.extend(iterable);
        set
    }
}

impl PartialEq for ObjectIdIndexSet {
    fn eq(&self, other: &ObjectIdIndexSet) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for ObjectIdIndexSet {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_id::ObjectIdManager;

    #[test]
    fn test_insertion_order_preserved() {
        let mut set = ObjectIdIndexSet::new();
        let id1 = ObjectId::from(ObjectIdManager::FIRST_OID);
        let id2 = ObjectId::from(ObjectIdManager::FIRST_OID + 1);
        let id3 = ObjectId::from(ObjectIdManager::FIRST_OID + 2);

        set.insert(id1);
        set.insert(id3);
        set.insert(id2);

        let collected: Vec<_> = set.iter().copied().collect();
        assert_eq!(collected, vec![id1, id3, id2]);
    }

    #[test]
    fn test_contains_and_removal() {
        let mut set = ObjectIdIndexSet::new();
        let id1 = ObjectId::from(ObjectIdManager::FIRST_OID);
        let id2 = ObjectId::from(ObjectIdManager::FIRST_OID + 1);

        set.insert(id1);
        set.insert(id2);

        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
        assert_eq!(set.len(), 2);

        assert!(set.shift_remove(&id1));
        assert!(!set.contains(&id1));
        assert!(set.contains(&id2));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_index_access() {
        let mut set = ObjectIdIndexSet::new();
        let id1 = ObjectId::from(ObjectIdManager::FIRST_OID);
        let id2 = ObjectId::from(ObjectIdManager::FIRST_OID + 1);

        set.insert(id1);
        set.insert(id2);

        assert_eq!(set.get_index(0), Some(&id1));
        assert_eq!(set.get_index(1), Some(&id2));
        assert_eq!(set.get_index(2), None);

        assert_eq!(set.get_full(&id1), Some((0, &id1)));
        assert_eq!(set.get_full(&id2), Some((1, &id2)));
    }
}
