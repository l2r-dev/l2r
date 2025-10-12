---@class MoveToEntity Helper module for MoveToEntity component operations
local MoveToEntity = {}

---Inserts a MoveToEntity component to make an entity move toward a target entity
---@param entity Entity The entity that should move
---@param target_entity Entity The target entity to move toward
---@param range number The desired distance to maintain from the target
function MoveToEntity.insert_component(entity, target_entity, range)
    local move_to_entity_data = construct(types.MoveToEntity, {
        target = target_entity,
        range = range
    })
    world.insert_component(entity, types.MoveToEntity, move_to_entity_data)
end

return MoveToEntity
