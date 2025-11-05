use bevy::{log, platform::collections::HashMap, prelude::*};
use bit_set::BitSet;

mod error;
mod id;
mod index_set;
mod query;

pub use error::*;
pub use id::*;
pub use index_set::*;
pub use query::*;

pub struct ObjectIdComponentsPlugin;
impl Plugin for ObjectIdComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectId>()
            .register_type::<ObjectIdManager>();
    }
}

#[derive(Clone, Component, Copy, Default)]
pub struct ObjectIdManagerTaskSpawned;

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct ObjectIdManager {
    #[reflect(ignore)]
    object_ids: BitSet,
    #[reflect(ignore)]
    entity_by_oid: HashMap<ObjectId, Entity>,
    free_id_count: usize,
    next_free_idx: usize,
    object_ids_len: usize,
}

impl Default for ObjectIdManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectIdManager {
    pub const FIRST_OID: usize = u16::MAX as usize * 4;
    pub const LAST_OID: usize = i32::MAX as usize;
    pub const INITIAL_CAPACITY: u64 = 100_000;
    pub const FREE_OBJECT_ID_SIZE: usize = Self::LAST_OID - Self::FIRST_OID;

    pub fn new() -> Self {
        let initial_capacity = const_primes::next_prime(Self::INITIAL_CAPACITY)
            .unwrap_or(Self::INITIAL_CAPACITY) as usize;
        let mut instance = Self {
            object_ids: BitSet::with_capacity(initial_capacity),
            free_id_count: Self::FREE_OBJECT_ID_SIZE,
            next_free_idx: 0,
            entity_by_oid: HashMap::with_capacity(initial_capacity),
            object_ids_len: initial_capacity,
        };
        instance.next_free_idx = 0;
        instance
    }

    pub fn prepare_occupied(occupied_ids: &[i32]) -> Self {
        let mut instance = Self::new();
        for id in occupied_ids {
            if (*id) >= Self::FIRST_OID as i32 {
                let id_index = *id - Self::FIRST_OID as i32;
                instance.object_ids.insert(id_index as usize);
                instance.free_id_count -= 1;
            } else {
                log::warn!(
                    "Warning: Failed to load object ID {} (< {})",
                    id,
                    Self::FIRST_OID
                );
            }
        }
        instance.next_free_idx = instance.find_next_clear_bit(0).unwrap_or_default();
        instance
    }

    pub fn register_entity(&mut self, entity: Entity, oid: ObjectId) {
        self.entity_by_oid.insert(oid, entity);
    }

    pub fn unregister_entity(&mut self, oid: ObjectId) {
        self.entity_by_oid.remove(&oid);
    }

    pub fn entity(&self, oid: ObjectId) -> Option<Entity> {
        self.entity_by_oid.get(&oid).copied()
    }

    pub fn entity_result(&self, oid: ObjectId) -> Result<Entity> {
        self.entity_by_oid
            .get(&oid)
            .copied()
            .ok_or_else(|| BevyError::from(format!("No entity found for object ID: {}", oid)))
    }

    pub fn release_id(&mut self, object_id: ObjectId) {
        let object_idx = usize::from(object_id);
        if object_idx >= Self::FIRST_OID {
            let id_index = object_idx - Self::FIRST_OID;
            self.object_ids.remove(id_index);
            self.free_id_count += 1;
            self.entity_by_oid.remove(&object_id);
            if id_index < self.next_free_idx {
                self.next_free_idx = id_index;
            }
        } else {
            log::warn!(
                "Warning: Failed to release object ID {} (< {})",
                object_id,
                Self::FIRST_OID
            );
        }
    }

    pub fn release_ids(&mut self, object_ids: &[ObjectId]) {
        for object_id in object_ids {
            self.release_id(*object_id);
        }
    }

    pub fn next_id(&mut self) -> ObjectId {
        let new_id = self.next_free_idx;
        self.object_ids.insert(new_id);
        self.free_id_count -= 1;

        let next_free = self.find_next_clear_bit(new_id);
        let next_free = match next_free {
            Some(id) => id,
            None => match self.find_next_clear_bit(0) {
                Some(id) => id,
                None => {
                    self.increase_capacity();
                    usize::from(self.next_id())
                }
            },
        };
        self.next_free_idx = next_free;
        ObjectId::from(new_id + Self::FIRST_OID)
    }

    fn find_next_clear_bit(&self, start_id: usize) -> Option<usize> {
        let capacity = self.object_ids.capacity();
        if start_id >= capacity {
            return None;
        }
        // Walk over *set* bits starting from `start`, searching for the first gap
        let mut expected = start_id;
        for used in self.object_ids.iter().filter(|&idx| idx >= start_id) {
            if used != expected {
                // we found a gap before the next used id
                return Some(expected);
            }
            expected += 1;
            if expected >= capacity {
                return None;
            }
        }

        if expected < capacity {
            return Some(expected);
        }
        None
    }

    pub fn used_id_count(&self) -> usize {
        (Self::LAST_OID - Self::FIRST_OID) - self.free_id_count
    }

    pub fn size(&self) -> usize {
        self.free_id_count
    }

    fn increase_capacity(&mut self) {
        let used_count = self.used_id_count();
        // Calculate 110% of used count to grow capacity with 10% overhead
        let target_capacity = (used_count * 11 / 10) as u64;
        let new_capacity =
            const_primes::next_prime(target_capacity).unwrap_or(target_capacity) as usize;
        let current_len = self.object_ids.get_ref().len();

        if new_capacity > current_len {
            self.object_ids
                .get_mut()
                .grow(new_capacity - current_len, false);
            self.free_id_count += new_capacity - current_len;
            self.object_ids_len = self.object_ids.get_ref().len();
        }

        let current_map_capacity = self.entity_by_oid.capacity();

        if new_capacity > current_map_capacity {
            self.entity_by_oid
                .reserve(new_capacity - current_map_capacity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let manager = ObjectIdManager::new();
        assert_eq!(manager.used_id_count(), 0);
        assert_eq!(manager.size(), ObjectIdManager::FREE_OBJECT_ID_SIZE);
    }

    #[test]
    fn test_next_id() {
        let mut manager = ObjectIdManager::new();
        let id = manager.next_id();

        assert_eq!(id, ObjectId::from(ObjectIdManager::FIRST_OID));

        assert_eq!(manager.used_id_count(), 1);
        assert_eq!(manager.size(), ObjectIdManager::FREE_OBJECT_ID_SIZE - 1);
    }

    #[test]
    fn test_release_id() {
        let mut manager = ObjectIdManager::new();
        let id = manager.next_id();
        assert_eq!(manager.used_id_count(), 1);

        manager.release_id(id);

        assert_eq!(manager.used_id_count(), 0);
        assert_eq!(manager.size(), ObjectIdManager::FREE_OBJECT_ID_SIZE);
    }

    #[test]
    fn test_reuse_released_id() {
        let mut manager = ObjectIdManager::new();
        let id1 = manager.next_id();
        manager.release_id(id1);
        let id2 = manager.next_id();

        assert_eq!(id1, id2);
    }

    // Take row of ids release few of them and check if the next id is correct
    #[test]
    fn test_reuse_released_id_in_row() {
        let mut manager = ObjectIdManager::new();
        let id1 = manager.next_id();
        let _id2 = manager.next_id();
        let id3 = manager.next_id();
        let _id4 = manager.next_id();

        assert_eq!(manager.used_id_count(), 4);

        manager.release_id(id1);
        manager.release_id(id3);

        let next_id = manager.next_id();

        assert_eq!(next_id, id1);
    }

    #[test]
    fn test_prepare_occupied() {
        let occupied = [
            ObjectIdManager::FIRST_OID,
            ObjectIdManager::FIRST_OID + 1,
            ObjectIdManager::FIRST_OID + 3,
            ObjectIdManager::FIRST_OID + 4,
        ];

        let occupied: Vec<i32> = occupied.iter().map(|&id| id as i32).collect();

        let mut manager = ObjectIdManager::prepare_occupied(&occupied);

        // The manager must report the pre‑occupied IDs.
        assert_eq!(manager.used_id_count(), occupied.len());
        assert_eq!(
            manager.size(),
            ObjectIdManager::FREE_OBJECT_ID_SIZE - occupied.len()
        );

        // FIRST_OID + 2 is the first unoccupied value.
        let next_id = manager.next_id();
        assert_eq!(next_id, ObjectId::from(ObjectIdManager::FIRST_OID + 2));

        // FIRST_OID + 5 is the next unoccupied value.
        let next_id = manager.next_id();
        assert_eq!(next_id, ObjectId::from(ObjectIdManager::FIRST_OID + 5));

        // After allocation the counters must be updated.
        assert_eq!(manager.used_id_count(), (occupied.len() + 2));
    }
}
