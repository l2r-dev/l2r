use super::ObjectId;
use bevy::{ecs::query::QueryEntityError, prelude::*};
use thiserror::Error;

/// Errors that can occur when querying entities by ObjectId
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ObjectIdQueryError {
    /// The ObjectId does not exist in the ObjectIdManager
    #[error("ObjectId {0} not found in ObjectIdManager")]
    ObjectIdNotFound(ObjectId),

    /// The ObjectId exists but the corresponding entity is invalid or missing
    #[error("ObjectId {0} maps to entity {1:?} which no longer exists")]
    EntityNotFound(ObjectId, Entity),

    /// The entity exists but doesn't match the query (wrong components, filtered out, etc.)
    #[error("ObjectId {0} maps to entity {1:?} but query failed: {2}")]
    QueryMismatch(ObjectId, Entity, #[source] QueryEntityError),
}

impl From<ObjectIdQueryError> for String {
    fn from(error: ObjectIdQueryError) -> Self {
        error.to_string()
    }
}
