local PassiveSkills = {}

local StatModifiers = require("data.scripts.game.StatModifiers")

-- Applies all stat modifiers defined in a passive skill
-- Uses format: "passive:{skill_id}:{stat}"
-- @param entity The entity to apply modifiers to
-- @param skill_definition The skill definition containing stat tables
-- @param level The skill level to use for values
function PassiveSkills.apply_stat_modifiers(entity, skill_definition, level)
    if not skill_definition.tables or not skill_definition.tables.stats then
        return
    end

    for stat, operation_data in pairs(skill_definition.tables.stats) do
        local source_key = "passive:" .. skill_definition.id .. ":" .. stat:lower()
        StatModifiers.apply_modifier_with_source(entity, stat, operation_data, level, source_key)
    end
end

-- Applies a specific stat modifier for a passive skill
-- Uses format: "passive:{skill_id}:{stat}"
-- @param entity The entity to apply the modifier to
-- @param skill_definition The skill definition
-- @param level The skill level to use for values
-- @param stat The stat to modify
-- @param operation_data The operation data (type and values array)
function PassiveSkills.apply_stat_modifier(entity, skill_definition, level, stat, operation_data)
    local source_key = "passive:" .. skill_definition.id .. ":" .. stat:lower()
    StatModifiers.apply_modifier_with_source(entity, stat, operation_data, level, source_key)
end

-- Removes all stat modifiers for a passive skill
-- Removes all modifiers containing "passive:{skill_id}"
-- @param entity The entity to remove modifiers from
-- @param skill_definition The skill definition
function PassiveSkills.remove_stat_modifiers(entity, skill_definition)
    local contains_string = "passive:" .. skill_definition.id
    StatModifiers.remove_modifiers_containing(entity, contains_string)
end

-- Removes a specific stat modifier for a passive skill
-- @param entity The entity to remove the modifier from
-- @param skill_definition The skill definition
-- @param stat The stat to remove the modifier for
function PassiveSkills.remove_stat_modifier(entity, skill_definition, stat)
    local source_key = "passive:" .. skill_definition.id .. ":" .. stat:lower()
    StatModifiers.remove_modifier_by_source(entity, source_key)
end

-- Conditionally applies or removes a stat modifier based on a condition
-- @param entity The entity to modify
-- @param skill_definition The skill definition
-- @param level The skill level
-- @param stat The stat to modify
-- @param operation_data The operation data
-- @param condition_met Whether the condition is met to apply the modifier
function PassiveSkills.apply_conditional_stat_modifier(entity, skill_definition, level, stat, operation_data,
                                                       condition_met)
    if condition_met then
        PassiveSkills.apply_stat_modifier(entity, skill_definition, level, stat, operation_data)
    else
        PassiveSkills.remove_stat_modifier(entity, skill_definition, stat)
    end
end

return PassiveSkills
