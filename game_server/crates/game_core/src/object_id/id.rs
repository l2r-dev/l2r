use crate::{
    character::Character, encounters::KnownEntitiesRemoved, items::Item, object_id::ObjectIdManager,
};
use bevy::{
    ecs::component::{ComponentHook, Immutable, StorageType},
    prelude::*,
};
// use bevy::ecs::component::{ComponentHooks, StorageType};
use l2r_core::model::generic_number::GenericNumber;
use scripting::prelude::ScriptValue;
use sea_orm::{
    TryFromU64, TryGetError, TryGetable, Value,
    sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
    str::FromStr,
};

/// ObjectId uniquely identifies game objects on the client side and in the database.
#[derive(
    Clone,
    Copy,
    Debug,
    Deref,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Reflect,
    Default,
    // Component,
)]
pub struct ObjectId(u32);
impl Component for ObjectId {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Immutable;

    fn on_add() -> Option<ComponentHook> {
        Some(|mut world, context| {
            let Some(object_id) = world.entity(context.entity).get::<Self>().copied() else {
                return;
            };
            let mut object_id_manager = world.resource_mut::<ObjectIdManager>();
            object_id_manager.register_entity(context.entity, object_id);
        })
    }

    fn on_despawn() -> Option<ComponentHook> {
        Some(|mut world, context| {
            let Some(object_id) = world.entity(context.entity).get::<Self>().copied() else {
                return;
            };

            world
                .commands()
                .trigger_targets(KnownEntitiesRemoved::new(object_id), context.entity);
        })
    }

    fn on_remove() -> Option<ComponentHook> {
        Some(|mut world, context| {
            let Some(object_id) = world.entity(context.entity).get::<Self>().copied() else {
                return;
            };
            // If entity has Item or Character component, unregister it, otherwise release it
            // because despawned Chars and Items are not removed from the database
            // and we need to keep their object id, but NPCs for example, not stored in the database
            // and we can release their object id for reuse
            let is_character = world.entity(context.entity).get::<Character>().is_some();
            let is_item = world.entity(context.entity).get::<Item>().is_some();
            let mut object_id_manager = world.resource_mut::<ObjectIdManager>();
            if is_character || is_item {
                object_id_manager.unregister_entity(object_id);
            } else {
                object_id_manager.release_id(object_id);
            }
        })
    }
}

impl ObjectId {
    pub fn test_data() -> Self {
        Self((ObjectIdManager::FIRST_OID - 1) as u32)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GenericNumber<u32> for ObjectId {
    fn value(&self) -> u32 {
        self.0
    }
}

impl From<ObjectId> for Value {
    fn from(id: ObjectId) -> Self {
        Value::Int(Some(id.0 as i32))
    }
}

impl TryGetable for ObjectId {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i32 = res.try_get_by_nullable(idx)?;
        Ok(ObjectId(value as u32))
    }
}

impl ValueType for ObjectId {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Int(Some(val)) => {
                if val >= 0 {
                    Ok(ObjectId(val as u32))
                } else {
                    Err(ValueTypeErr)
                }
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(ObjectId).to_owned()
    }

    fn column_type() -> ColumnType {
        ColumnType::Integer
    }

    fn array_type() -> ArrayType {
        ArrayType::Int
    }
}

impl TryFromU64 for ObjectId {
    fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
        if n > u32::MAX as u64 {
            return Err(sea_orm::DbErr::Type(format!(
                "ObjectId value cannot be greater than {}: {}",
                u32::MAX,
                n
            )));
        }
        Ok(ObjectId(n as u32))
    }
}

impl Nullable for ObjectId {
    fn null() -> Value {
        Value::Int(None)
    }
}

impl TryFrom<&ScriptValue> for ObjectId {
    type Error = ();

    fn try_from(value: &ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::Integer(n) if *n >= 0 && *n <= u32::MAX as i64 => Ok(ObjectId(*n as u32)),
            ScriptValue::Float(n) if *n >= 0.0 && *n <= u32::MAX as f64 => Ok(ObjectId(*n as u32)),
            _ => Err(()),
        }
    }
}

l2r_core::impl_std_math_operations!(ObjectId, u32);
l2r_core::impl_primitive_conversions!(ObjectId, u32);
