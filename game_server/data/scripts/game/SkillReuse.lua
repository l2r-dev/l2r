local Logger = req("data.scripts.Logger")
Logger.set_script_name('game.SkillReuse')

---@class SkillReuse
local SkillReuse = {}

-- Starts a reuse timer for a skill
-- @param entity The entity using the skill
-- @param skill_ref The skill reference containing the skill ID
-- @param reuse_delay_ms The reuse delay in milliseconds
function SkillReuse.start_timer(entity, skill_ref, reuse_delay_ms)
    -- Only start timer if there's actually a reuse delay
    if reuse_delay_ms == nil or reuse_delay_ms == 0 then
        return
    end

    -- Get or create the SkillReuseTimers component
    local reuse_timers = world.get_component(entity, types.SkillReuseTimers)
    if not reuse_timers then
        -- Component doesn't exist, this should not happen as it's added with SkillList
        return
    end

    -- Create a new Timer using the global Timer constructor
    local timer_mode = construct(types.TimerMode, { variant = "Once" })
    local duration = Duration.from_millis(reuse_delay_ms)
    local timer = Timer.new(duration, timer_mode)

    -- Insert the timer into the HashMap
    reuse_timers._1[skill_ref.id] = timer
end

-- Checks if a skill is on cooldown
-- @param entity The entity to check
-- @param skill_id The skill ID to check
-- @return boolean True if the skill is on cooldown, false otherwise
function SkillReuse.is_on_cooldown(entity, skill_id)
    local reuse_timers = world.get_component(entity, types.SkillReuseTimers)
    if not reuse_timers then
        return false
    end

    -- Check if timer exists in the HashMap
    local timer = reuse_timers._1[skill_id]
    if not timer then
        return false
    end

    -- Call finished() as a method, not a field
    return not timer:finished()
end

-- Gets the remaining cooldown time in milliseconds
-- @param entity The entity to check
-- @param skill_id The skill ID to check
-- @return number The remaining cooldown time in milliseconds, or 0 if not on cooldown
function SkillReuse.remaining_cooldown_ms(entity, skill_id)
    local reuse_timers = world.get_component(entity, types.SkillReuseTimers)
    if not reuse_timers then
        return 0
    end

    -- Get timer from HashMap
    local timer = reuse_timers._1[skill_id]
    if not timer or timer:finished() then
        return 0
    end

    -- Use Timer's remaining() method to get remaining duration
    local remaining_duration = timer:remaining()
    local remaining_ms = remaining_duration:as_millis()

    return remaining_ms
end

-- Clears a specific skill's cooldown
-- @param entity The entity to modify
-- @param skill_id The skill ID to clear
function SkillReuse.clear_cooldown(entity, skill_id)
    local reuse_timers = world.get_component(entity, types.SkillReuseTimers)
    if not reuse_timers then
        return
    end

    -- Remove timer from HashMap
    reuse_timers._1:remove(skill_id)
end

-- Clears all skill cooldowns for an entity
-- @param entity The entity to modify
function SkillReuse.clear_all_cooldowns(entity)
    local reuse_timers = world.get_component(entity, types.SkillReuseTimers)
    if not reuse_timers then
        return
    end

    -- Clear the entire HashMap
    reuse_timers._1:clear()
end

return SkillReuse
