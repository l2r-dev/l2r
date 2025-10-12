---@class InActionPathfindingTimer
local InActionPathfindingTimer = {}

---Inserts an InActionPathfindingTimer component to prevent repeated pathfinding requests
---@param entity Entity The entity that should have the cooldown timer
function InActionPathfindingTimer.insert_component(entity)
    -- Create timer with 1 second cooldown (matches PATHFINDING_COOLDOWN in Rust)
    local timer_mode = construct(types.TimerMode, { variant = "Once" })
    local duration = Duration.from_millis(1000)
    local timer = Timer.new(duration, timer_mode)

    local timer_component = construct(types.InActionPathfindingTimer, { _1 = timer })
    world.insert_component(entity, types.InActionPathfindingTimer, timer_component)
end

---Checks if an entity has an active pathfinding cooldown timer
---@param entity Entity The entity to check
---@return boolean has_timer True if the entity has the cooldown timer
function InActionPathfindingTimer.has_component(entity)
    local timer = world.get_component(entity, types.InActionPathfindingTimer)
    return timer ~= nil
end

return InActionPathfindingTimer
