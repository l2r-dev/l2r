require("data.scripts.Utils")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("game.PaperDoll")


local PaperDoll = {}

-- Global table to store last change ticks for each entity
local last_change_ticks = {}

local weapons_with_extra_kind = {
    "Sword",
    "Blunt",
    "Dagger",
    "Fist",
}

--[[
    Checks if an entity's PaperDoll has changed since the last check
    @param entity The entity to check
    @return true if PaperDoll has changed, false otherwise
]]
function PaperDoll.is_changed(entity)
    local entity_key = tostring(entity:index())
    local component_ticks = world.get_component_ticks(entity, PaperDoll.type())

    -- Get the last change tick for this entity, or create a new one if it doesn't exist
    local last_tick = last_change_ticks[entity_key]
    if not last_tick then
        last_tick = construct(types.Tick, { tick = 0 })
        last_change_ticks[entity_key] = last_tick
    end

    -- Check if the PaperDoll has changed
    local is_changed = component_ticks:is_changed(last_tick, component_ticks.changed)

    -- Update the last change tick for this entity if it has changed
    if is_changed then
        last_change_ticks[entity_key] = component_ticks.changed
    end

    return is_changed
end

function PaperDoll.update_change_tick(entity)
    local entity_key = tostring(entity:index())
    local component_ticks = world.get_component_ticks(entity, PaperDoll.type())
    last_change_ticks[entity_key] = component_ticks.changed
end

--[[
    Gets the equipped item in a specific slot

    @param entity The entity to check
    @param slot_variant The slot variant name ("RightHand", "LeftHand", etc.)
    @return The equipped item and item info, or nil if no item equipped
]]
function PaperDoll.get_equipped_item(entity, slot_variant)
    local paperdoll = world.get_component(entity, types.PaperDoll)
    if not paperdoll then
        return nil
    end
    local slot = construct(types.DollSlot, { variant = slot_variant })
    local unique_item = paperdoll._1[slot]
    if not unique_item then
        return nil
    end
    local item = unique_item._2
    local items_data_table = world.get_resource(types.ItemsDataTable)._1
    local item_handle = items_data_table[item.id]
    local retrieved_asset = world.get_asset(item_handle, types.ItemsInfo)
    local item_info = retrieved_asset._1[item.id]
    return item, item_info
end

--[[
    Gets the weapon type of the equipped weapon

    @param entity The entity to check
    @param slot_variant The slot variant name (usually "RightHand")
    @return Two values: the default weapon kind as a string, and the extra kind if applicable, or nil if no weapon equipped
]]
function PaperDoll.get_weapon_kind(entity, slot_variant)
    local item, item_info = PaperDoll.get_equipped_item(entity, slot_variant)

    if not item or not item_info then
        return nil, nil
    end

    local default_kind = item_info.kind._1.kind:variant_name()
    local extra_kind = nil

    if contains(weapons_with_extra_kind, default_kind) then
        extra_kind = item_info.kind._1.kind._1:variant_name()
    end

    return default_kind, extra_kind
end

--[[
    Gets the armor kind of the equipped item

    @param entity The entity to check
    @param slot_variant The slot variant name ("Chest", "Legs", (DollSlot))
    @return The armor kind as a string, or nil if no armor equipped
]]
function PaperDoll.get_armor_kind(entity, slot_variant)
    local item, item_info = PaperDoll.get_equipped_item(entity, slot_variant)

    if not item or not item_info then
        return nil
    end
    local armor_kind = item_info.kind._1:variant_name()

    return armor_kind
end

--[[
    Checks if the entity has a required weapon type equipped

    @param entity The entity to check
    @param required_weapons A table of weapon type strings that fulfill the requirement
    @param required_extra_kinds A table of extra weapon type strings that fulfill the requirement (optional)
    @return true if the equipped weapon matches one of the required types, false otherwise
]]
function PaperDoll.has_required_weapon(entity, required_weapons, required_extra_kinds)
    local slot_variant = "RightHand"
    local weapon_kind, extra_kind = PaperDoll.get_weapon_kind(entity, slot_variant)
    if not weapon_kind then
        return false
    end
    -- Check if the main weapon kind is in the required list
    if contains(required_weapons, weapon_kind) then
        -- If this weapon has an extra kind and extra kinds are required
        if contains(weapons_with_extra_kind, weapon_kind) and required_extra_kinds then
            -- Check if the extra kind matches requirements
            return contains(required_extra_kinds, extra_kind)
        end
        -- If no extra kind is needed or the weapon doesn't have an extra kind category
        return true
    end
    return false
end

--[[
    Checks if the entity has one of the required armor types equipped in both the Chest and Legs slots

    @param entity The entity to check
    @param required_armor_kinds A string or table of armor type strings that fulfill the requirement
    @return true if both slots have the same required armor type from the list, false otherwise
]]
function PaperDoll.has_required_armor(entity, required_armor_kinds)
    local slots_to_check = { "Chest", "Legs" }
    -- Convert single string to table for consistent processing
    if type(required_armor_kinds) == "string" then
        required_armor_kinds = { required_armor_kinds }
    end
    -- Get the armor kind from the chest slot
    local chest_armor_kind = PaperDoll.get_armor_kind(entity, "Chest")
    if not chest_armor_kind then
        return false
    end
    -- Check if chest armor is one of the required types
    local is_valid_armor = false
    for _, required_kind in ipairs(required_armor_kinds) do
        if chest_armor_kind == required_kind then
            is_valid_armor = true
            break
        end
    end
    if not is_valid_armor then
        return false
    end
    -- Check if all other slots have the same armor kind as the chest
    for i = 2, #slots_to_check do
        local armor_kind = PaperDoll.get_armor_kind(entity, slots_to_check[i])
        if armor_kind ~= chest_armor_kind then
            return false
        end
    end

    return true
end

--[[
    Checks if the entity has a shield equipped
    @param entity The entity to check
    @return true if a shield is equipped, false otherwise
]]
function PaperDoll.has_shield(entity)
    local item, item_info = PaperDoll.get_equipped_item(entity, "LeftHand")
    if not item or not item_info then
        return false
    end
    return item_info.kind._1:variant_name() == "Shield"
end

function PaperDoll.type()
    return types.PaperDoll
end

return PaperDoll
