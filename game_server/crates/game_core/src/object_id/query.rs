use super::{ObjectId, ObjectIdManager, ObjectIdQueryError};
use bevy::{
    ecs::query::{QueryData, QueryFilter, ROQueryItem},
    prelude::*,
};

pub trait QueryByObjectId<'w, 's, Q: QueryData, F: QueryFilter> {
    fn by_object_id(
        &self,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<ROQueryItem<'_, Q>, ObjectIdQueryError>;
}

pub trait QueryByObjectIdMut<'w, 's, Q: QueryData, F: QueryFilter> {
    fn by_object_id_mut(
        &mut self,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<<Q as QueryData>::Item<'_>, ObjectIdQueryError>;
}

pub trait QueryStateByObjectId<Q: QueryData, F: QueryFilter> {
    fn by_object_id<'w>(
        &mut self,
        world: &'w World,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<ROQueryItem<'w, Q>, ObjectIdQueryError>;
}

pub trait QueryStateByObjectIdMut<Q: QueryData, F: QueryFilter> {
    fn by_object_id_mut<'w>(
        &mut self,
        world: &'w mut World,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<<Q as QueryData>::Item<'w>, ObjectIdQueryError>;
}

impl<'w, 's, Q: QueryData, F: QueryFilter> QueryByObjectId<'w, 's, Q, F> for Query<'w, 's, Q, F> {
    #[inline]
    fn by_object_id(
        &self,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<ROQueryItem<'_, Q>, ObjectIdQueryError> {
        let entity = manager
            .entity(id)
            .ok_or(ObjectIdQueryError::ObjectIdNotFound(id))?;

        self.get(entity)
            .map_err(|query_error| ObjectIdQueryError::QueryMismatch(id, entity, query_error))
    }
}

impl<'w, 's, Q: QueryData, F: QueryFilter> QueryByObjectIdMut<'w, 's, Q, F>
    for Query<'w, 's, Q, F>
{
    #[inline]
    fn by_object_id_mut(
        &mut self,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<<Q as QueryData>::Item<'_>, ObjectIdQueryError> {
        let entity = manager
            .entity(id)
            .ok_or(ObjectIdQueryError::ObjectIdNotFound(id))?;

        self.get_mut(entity)
            .map_err(|query_error| ObjectIdQueryError::QueryMismatch(id, entity, query_error))
    }
}

impl<Q: QueryData, F: QueryFilter> QueryStateByObjectId<Q, F> for QueryState<Q, F> {
    #[inline]
    fn by_object_id<'w>(
        &mut self,
        world: &'w World,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<ROQueryItem<'w, Q>, ObjectIdQueryError> {
        let entity = manager
            .entity(id)
            .ok_or(ObjectIdQueryError::ObjectIdNotFound(id))?;

        self.get(world, entity)
            .map_err(|query_error| ObjectIdQueryError::QueryMismatch(id, entity, query_error))
    }
}

impl<Q: QueryData, F: QueryFilter> QueryStateByObjectIdMut<Q, F> for QueryState<Q, F> {
    #[inline]
    fn by_object_id_mut<'w>(
        &mut self,
        world: &'w mut World,
        id: ObjectId,
        manager: &ObjectIdManager,
    ) -> Result<<Q as QueryData>::Item<'w>, ObjectIdQueryError> {
        let entity = manager
            .entity(id)
            .ok_or(ObjectIdQueryError::ObjectIdNotFound(id))?;

        self.get_mut(world, entity)
            .map_err(|query_error| ObjectIdQueryError::QueryMismatch(id, entity, query_error))
    }
}
