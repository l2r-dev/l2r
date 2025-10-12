local Transform = {}

-- Global table to store last change ticks for each entity
local last_change_ticks = {}

-- Function to check if an entity's transform has changed since the last check
-- @param entity - The entity to check
-- @return boolean - true if transform has changed, false otherwise
function Transform.is_changed(entity)
    local entity_key = tostring(entity:index())
    local component_ticks = world.get_component_ticks(entity, Transform.type())

    -- Get the last change tick for this entity, or create a new one if it doesn't exist
    local last_tick = last_change_ticks[entity_key]
    if not last_tick then
        last_tick = construct(types.Tick, { tick = 0 })
        last_change_ticks[entity_key] = last_tick
    end

    -- Check if the transform has changed
    local is_changed = component_ticks:is_changed(last_tick, component_ticks.changed)

    -- Update the last change tick for this entity if it has changed
    if is_changed then
        last_change_ticks[entity_key] = component_ticks.changed
    end

    return is_changed
end

function Transform.type()
    return types.Transform
end

return Transform
