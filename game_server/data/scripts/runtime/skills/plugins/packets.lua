require("data.scripts.Utils")
local SkillList = req("data.scripts.game.SkillList")
local Logger = req("data.scripts.Logger")
local Stats = req("data.scripts.game.Stats")
Logger.set_script_name("skills.packets")
local MagicSkillCanceled = req("data.scripts.packets.MagicSkillCanceled")
local SkillReuse = req("data.scripts.game.SkillReuse")
local SystemMessage = req("data.scripts.packets.SystemMessage")

---@class SkillPackets
local packets = {}

---@param entity table The entity object
---@param packet table The packet data
---@param skills_storage table The skills storage
local function handle_enter_world(entity, packet, skills_storage)
    if not skills_storage then
        Logger.error("- EnterWorld - SkillsStorage not available - cannot process skill packet")
        return
    end
    local char_id = world.get_component(entity, types.ObjectId)
    local character_skills = SkillList.load_skills_from_db(char_id)

    local sub_class = world.get_component(entity, types.SubClass)
    local class_id = sub_class or world.get_component(entity, types.BaseClass)
    if not class_id then
        Logger.error("Entity " .. entity:index() .. " has no BaseClass or SubClass")
        return
    end

    local skill_trees_table = world.get_resource(types.SkillTreesHandlers)._1
    local tree_handle = skill_trees_table[class_id._1]
    if not tree_handle then
        Logger.error("No skill tree found for class ID " .. tostring(class_id._1))
        return
    end
    local skill_tree = world.get_asset(tree_handle, types.SkillTree)
    if not skill_tree then
        Logger.error("Failed to load skill tree asset for class ID " .. tostring(class_id._1))
        return
    end
    local class_tree_nodes = skill_tree._1
    local skill_list = world.get_component(entity, types.SkillList)

    -- If no skills in DB, grant auto-skills for character's level
    if table_count(character_skills) == 0 then
        local char_level = Stats.get(entity, "ProgressLevelStats", "Level")

        -- Get sub_class variant for database storage
        if not sub_class then
            Logger.error("Entity " .. entity:index() .. " has no SubClass component - cannot grant auto-skills")
            return
        end
        local sub_class_variant_name = sub_class:variant_name()
        local sub_class_variant = construct(types.SubClassVariant, { variant = sub_class_variant_name })

        -- Grant and save auto-skills
        local count = SkillList.grant_auto_skills(entity, char_id, skill_list, class_tree_nodes, char_level,
            sub_class_variant, skills_storage)
        Logger.info("Granted " .. tostring(count) .. " auto-skills for level " .. tostring(char_level))
    else
        -- Load existing skills from database
        SkillList.load_skills_to_list(skill_list, character_skills, skills_storage)
    end
end

---@param entity table The entity object
---@param packet table The packet data
local function handle_cancel_target(entity, packet)
    local casting = world.get_component(entity, types.LuaCasting)
    if casting then
        world.remove_component(entity, types.LuaCasting)
        MagicSkillCanceled.send(entity)
    end

    local pending_skill = world.get_component(entity, types.LuaPendingSkill)
    if pending_skill then
        world.remove_component(entity, types.LuaPendingSkill)
    end
end

---@param entity table The entity object
---@param _packet table The packet data
local function handle_move_backward_to_location(entity, _packet)
    local pending_skill = world.get_component(entity, types.LuaPendingSkill)
    if pending_skill then
        world.remove_component(entity, types.LuaPendingSkill)
    end
end

---@param entity table The entity object
---@param packet table The packet data
---@param skills_storage table The skills storage
local function handle_magic_skill_use(entity, packet, skills_storage)
    if not skills_storage then
        Logger.error("SkillsStorage not available - cannot process skill packet")
        return
    end

    -- Check if already casting - if so, check if we already have a queued skill
    local is_casting = world.get_component(entity, types.LuaCasting)
    local has_pending = world.get_component(entity, types.LuaPendingSkill)

    if is_casting and has_pending then
        -- Already casting AND already have a skill queued - can't queue another one
        return
    end

    -- Check for other active actions that should block skill use (not casting-related)
    local active_action = world.get_component(entity, types.ActiveAction)
    if active_action and not is_casting then
        -- Has active actions but not casting - this is some other action, block it
        return
    end

    local skill_list = world.get_component(entity, types.SkillList)
    local skill_ref = skill_list._1[packet._1.skill_id]

    if not skill_ref then
        Logger.warn("Entity " ..
            entity:index() .. " attempted to use unknown skill ID: " .. tostring(packet._1.skill_id._1))
        return
    end

    if skill_ref.disabled then
        SystemMessage.send_simple(entity, 158)
        return
    end

    local handler = skills_storage.get(skill_ref.id._1)
    if not handler then
        Logger.error("Skill ID " .. tostring(skill_ref.id._1) .. " not found in SkillsStorage")
        return
    end

    -- Check if skill is on cooldown
    if SkillReuse.is_on_cooldown(entity, skill_ref.id) then
        -- Skill is on cooldown, send system message with skill name
        local skill_name = handler.definition.name or "Unknown Skill"
        SystemMessage.send_with_text(entity, 48, skill_name)
        return
    end

    handler.pend(entity, skill_ref, packet._1.shift_pressed, packet._1.ctrl_pressed)
end

local packet_handlers = {
    EnterWorld = handle_enter_world,
    RequestCancelTarget = handle_cancel_target,
    RequestMagicSkillUse = handle_magic_skill_use,
    MoveBackwardToLocation = handle_move_backward_to_location
}

---@param entity table The entity object
---@param packet table The packet data
---@param skills_storage table The skills storage
function packets.on_packet_received(entity, packet, skills_storage)
    local handler = packet_handlers[packet:variant_name()]
    if handler then
        handler(entity, packet, skills_storage)
    end
end

return packets
