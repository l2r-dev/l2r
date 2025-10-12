require("data.scripts.Utils")
local Logger = req("data.scripts.Logger")


local Casting = {}

-- Inserts a Casting component to mark an entity as casting a skill
-- @param caster_entity The entity performing the casting
-- @param target_entity The target of the casting
-- @param casting_time The time in milliseconds for the cast to complete (optional, defaults to 0)
-- @param skill The skill being cast
function Casting.insert_component(caster_entity, target_entity, casting_time, skill_ref)
    local casting_time = casting_time or 0
    local casting_data = {
        target = target_entity,
        duration = casting_time,
        elapsed_time = 0.0,
        skill_ref = skill_ref,
        launched = false,
    }

    -- Add LuaCasting component to entity
    world.insert_component(caster_entity, types.LuaCasting, construct(types.DynamicComponent, {
        data = casting_data
    }))
    -- Stop any movement the caster is doing before casting
    world.remove_component(caster_entity, types.MoveTarget)

    -- Add AnimationTimer component to manage animation timing (automatically adds Animation component due to #[require])
    local timer_mode_variant = construct(types.TimerMode, { variant = "Repeating" })
    local duration = Duration.from_millis(casting_time)
    local timer = Timer.new(duration, timer_mode_variant)

    local animation_timer = construct(types.AnimationTimer, { _1 = timer })
    world.insert_component(caster_entity, types.AnimationTimer, animation_timer)
    Logger.trace("Added AnimationTimer component with duration " ..
        tostring(casting_time) .. "s to entity " .. tostring(caster_entity))

    -- Remove LuaPendingSkill component when casting starts to ensure clean state transition
    world.remove_component(caster_entity, types.LuaPendingSkill)
end

-- Checks if an entity is currently casting
-- @param caster_entity The entity to check
-- @return true if the entity is casting, false otherwise
function Casting.in_progress(caster_entity)
    local casting = world.get_component(caster_entity, types.LuaCasting)
    if casting then
        return true
    end
    return false
end

return Casting
