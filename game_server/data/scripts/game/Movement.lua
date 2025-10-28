---@class Movement Helper module for Movement component operations
local Movement = {}

---Inserts a Movement component to make an entity move toward a target entity
---@param entity Entity The entity that should move
---@param target_entity Entity The target entity to move toward
---@param range number The desired distance to maintain from the target
function Movement.to_entity(entity, target_entity, range)
    local movement_data = construct(types.Movement, {
        variant = "ToEntity",
        target = target_entity,
        range = range
    })
    world.insert_component(entity, types.Movement, movement_data)
end

---Checks if an entity has Movement component and is moving to a specific target
---@param entity Entity The entity to check
---@param target_entity Entity The target entity to compare
---@return boolean true if moving to the target entity
function Movement.is_moving_to_entity(entity, target_entity)
    local movement = world.get_component(entity, types.Movement)
    if movement == nil then
        return false
    end

    -- Check if it's ToEntity variant
    local variant = movement:variant_name()
    if variant == "ToEntity" then
        -- Access the ToEntity variant fields
        return movement.target == target_entity
    end

    return false
end ---Removes the Movement component from an entity

---@param entity Entity The entity to remove movement from
function Movement.remove(entity)
    world.remove_component(entity, types.Movement)
end

return Movement
