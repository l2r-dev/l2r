local StatModifiers = {}

local Stats = require("data.scripts.game.Stats")

-- Applies a single stat modifier with a prebuilt source key
---@param entity Entity The entity to apply the modifier to
---@param stat string The stat to modify (e.g., "PAtk", "PDef")
---@param operation_data table Array containing [operation_type, values_array]
---@param level number The level to use for indexing into the values array
---@param source_key string The unique source key for tracking this modifier
---@param priority? number The priority of the modifier (optional, defaults to 1)
function StatModifiers.apply_modifier_with_source(entity, stat, operation_data, level, source_key, priority)
    local stat_modifiers = world.get_component(entity, types.StatModifiers)

    local operation_type = operation_data[1] -- e.g., "Add" or "Mul"
    local values = operation_data[2]         -- e.g., {9, 11, 12, 13, 14}
    local value = values[level]              -- Get the value for the current level

    local operation = construct(types["StatsOperation<f32>"], { variant = operation_type, _1 = value })

    local priority = priority or 1

    local modifier = construct(types.StatModifier, {
        stat = Stats.kind(stat),
        operation = operation,
        priority = priority
    })

    stat_modifiers._1[source_key] = modifier
end

-- Removes a stat modifier by exact source key
---@param entity Entity The entity to remove the modifier from
---@param source_key string The exact source key of the modifier to remove
function StatModifiers.remove_modifier_by_source(entity, source_key)
    local stat_modifiers = world.get_component(entity, types.StatModifiers)
    stat_modifiers._1:remove(source_key)
end

-- Removes all modifiers that contain a specific string in their source key
---@param entity Entity The entity to remove modifiers from
---@param contains_string string The string that must be contained in the source key
function StatModifiers.remove_modifiers_containing(entity, contains_string)
    local stat_modifiers = world.get_component(entity, types.StatModifiers)
    local keys_to_remove = {}

    -- Collect keys that contain the string
    for key, _ in pairs(stat_modifiers._1) do
        if string.find(key, contains_string, 1, true) then
            table.insert(keys_to_remove, key)
        end
    end

    -- Remove the collected keys
    for _, key in ipairs(keys_to_remove) do
        stat_modifiers._1:remove(key)
    end
end

return StatModifiers
