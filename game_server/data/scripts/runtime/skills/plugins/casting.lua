require("data.scripts.Utils")
local Logger = req("data.scripts.Logger")
local Stats = req("data.scripts.game.Stats")
local SkillList = req("data.scripts.game.SkillList")
local PaperDoll = req("data.scripts.game.PaperDoll")
local StatModifiers = req("data.scripts.game.StatModifiers")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")
local SystemMessage = req("data.scripts.packets.SystemMessage")
local MagicSkillCanceled = req("data.scripts.packets.MagicSkillCanceled")
Logger.set_script_name("skills.casting")

---@type SkillsStorage Skills storage reference
local skills_storage = {}

-- System function for processing casting entities
---@param time_ref Time bevy Time<()> resource ReflectReference
---@param casting_query Query Query result containing casting entities
function CASTING_SYSTEM(time_ref, casting_query)
    local delta_ms = time_ref.delta_secs * 1000.0
    local processed_count = 0

    if casting_query and #casting_query > 0 then
        for i, result in pairs(casting_query) do
            local entity = result:entity()
            local components = result:components()

            if #components > 0 then
                processed_count = processed_count + 1
                -- Manual timer handling - get the component directly from world for mutation
                local casting_component = components[1]
                local current_elapsed = casting_component.data.elapsed_time
                local new_elapsed = current_elapsed + delta_ms
                local skill_ref = casting_component.data.skill_ref
                local casting_target = casting_component.data.target

                -- Check if casting is finished
                if new_elapsed >= casting_component.data.duration then
                    -- Check if target is still alive before launching (prevents casting on dead targets)
                    local can_launch = true
                    if casting_target and casting_target ~= entity then
                        local target_dead = world.get_component(casting_target, types.Dead)
                        local target_hp = Stats.get(casting_target, "VitalsStats", "Hp")
                        if target_dead or target_hp <= 0 then
                            can_launch = false
                            -- Send invalid target message and cancel the casting animation on client
                            SystemMessage.send(entity, 109, {}) -- Invalid target
                            MagicSkillCanceled.send(entity)
                        end
                    end

                    if can_launch then
                        local handler = skills_storage.get(skill_ref.id._1)
                        if handler and handler.launch then
                            handler.launch(entity, casting_target, skill_ref)
                        end
                    end

                    world.remove_component(entity, types.LuaCasting)
                else
                    local updated_data = {
                        target = casting_component.data.target,
                        duration = casting_component.data.duration,
                        elapsed_time = new_elapsed,
                        skill_ref = casting_component.data.skill_ref,
                    }
                    world.insert_component(entity, types.LuaCasting, construct(types.DynamicComponent, {
                        data = updated_data
                    }))
                end
            end
        end
    end
end

-- System function for processing pending skill entities
---@param pending_query table Query result containing pending skill entities
function PENDING_SKILL_SYSTEM(pending_query)
    if pending_query and #pending_query > 0 then
        for i, result in pairs(pending_query) do
            local entity = result:entity()
            local components = result:components()

            if #components > 0 then
                ---@type DynamicComponent
                local lua_pending = components[1] -- LuaPendingSkill should be first
                ---@type LuaPendingSkillData
                local lua_pending_data = lua_pending.data
                local skill_id = lua_pending_data.skill_ref.id._1
                local skill_level = lua_pending_data.skill_ref.level._1

                -- Check if entity is currently casting
                local casting_component = world.get_component(entity, types.LuaCasting)

                if casting_component then
                    -- Entity is casting, but check if target is still valid
                    local target_entity = lua_pending_data.target_entity
                    if target_entity and target_entity ~= entity then
                        -- Check if target is dead or HP=0 - if so, cancel the queued skill
                        local target_dead = world.get_component(target_entity, types.Dead)
                        local target_hp = Stats.get(target_entity, "VitalsStats", "Hp")

                        if target_dead or target_hp <= 0 then
                            world.remove_component(entity, types.LuaPendingSkill)
                        end
                    end
                else
                    -- No active cast, validate target and process the pending skill
                    local target_entity = lua_pending_data.target_entity
                    local target_valid = true

                    -- Check if target is still valid (not dead or HP=0, unless it's self-target)
                    if target_entity and target_entity ~= entity then
                        local target_dead = world.get_component(target_entity, types.Dead)
                        local target_hp = Stats.get(target_entity, "VitalsStats", "Hp")

                        if target_dead or target_hp <= 0 then
                            target_valid = false
                        end
                    end

                    if not target_valid then
                        -- Target is dead, cancel the pending skill
                        world.remove_component(entity, types.LuaPendingSkill)
                    else
                        local handler = skills_storage.get(skill_id)
                        if not handler then
                            Logger.error("Skill ID " ..
                                tostring(skill_id) .. " not found in SkillsStorage")
                            world.remove_component(entity, types.LuaPendingSkill)
                            return
                        end
                        handler.on_pending(entity, lua_pending_data)
                    end
                end
            end
        end
    end
end

function SKILL_LIST_UPDATES(skill_lists_query)
    if not skill_lists_query or #skill_lists_query == 0 then
        return
    end

    for i, result in pairs(skill_lists_query) do
        local entity = result:entity()
        local components = result:components()

        if #components < 2 then
            goto continue
        end

        -- Check if either PaperDoll or SkillList has changed
        local paperdoll_changed = PaperDoll.is_changed(entity)
        local skilllist_changed = SkillList.is_changed(entity)

        if not paperdoll_changed and not skilllist_changed then
            goto continue
        end

        -- Get all skills from the skill list
        local skill_list = world.get_component(entity, types.SkillList)
        local entered_world = world.get_component(entity, types.EnteredWorld)

        -- Send skill list to client when it changes
        if skilllist_changed and entered_world then
            -- First, sync skills to database
            local char_id = world.get_component(entity, types.ObjectId)
            local sub_class = world.get_component(entity, types.SubClass)
            if char_id and sub_class then
                -- Get sub_class variant for database storage
                local sub_class_variant_name = sub_class:variant_name()
                local sub_class_variant = construct(types.SubClassVariant, { variant = sub_class_variant_name })
                -- Sync to database and preload skill handlers
                SkillList.sync_skills_to_db(char_id, skill_list, sub_class_variant, skills_storage)
            end
            -- Then send the updated skill list to client
            local gs_packet = construct(types.GameServerPacket, {
                variant = "SkillList",
                _1 = skill_list
            })
            GameServerPacket.send({ entity, gs_packet })
        end
        -- Clean up all existing passive modifiers before applying new ones
        StatModifiers.remove_modifiers_containing(entity, "passive:")

        -- Iterate through entity's skill list and apply passive skills
        for skill_id_obj, skill_ref in pairs(skill_list._1) do
            if skill_ref.kind:variant_name() == "Passive" then
                local skill_id = skill_id_obj._1
                local handler = skills_storage.get(skill_id)
                if handler and handler.definition and
                    handler.definition.kind == "Passive" and
                    handler.apply_passive
                then
                    handler.apply_passive(entity, skill_ref)
                end
            end
        end

        ::continue::
    end
end

-- System function for processing abnormal effects changes
---@param abnormal_query Query Query result containing entities with AbnormalEffects
function ABNORMAL_EFFECTS_SYSTEM(abnormal_query)
    if not skills_storage or not abnormal_query or #abnormal_query == 0 then
        return
    end

    for _, result in pairs(abnormal_query) do
        local entity = result:entity()
        if AbnormalEffects.is_changed(entity) then
            AbnormalEffects.process_abnormal_changes(entity, skills_storage)
        end
    end
end

---@class SkillCasting
local Casting = {}

function Casting.create_components()
    ---@class LuaCastingData
    ---@field target Entity Target entity being casted upon
    ---@field duration number Total casting duration in milliseconds
    ---@field elapsed_time number Elapsed casting time in milliseconds
    ---@field skill_ref Skill Reference to the skill being casted
    ---@field launched boolean Whether the skill has been launched

    types.LuaCasting = register_dyn_component("LuaCasting")

    ---@class LuaPendingSkillData
    ---@field caster_entity Entity The entity casting the skill
    ---@field target_entity Entity The target entity of the skill
    ---@field skill_ref Skill Reference to the skill being casted

    types.LuaPendingSkill = register_dyn_component("LuaPendingSkill")
end

---@param skills_stor SkillsStorage
function Casting.create_systems(skills_stor)
    skills_storage = skills_stor
    -- Check if systems already exist to prevent duplicates on hot reload
    local update_schedule = world.get_schedule_by_name("Update")
    if update_schedule:get_system_by_name("CASTING_SYSTEM") then
        Logger.debug("Casting systems already exist, skipping creation...")
        return
    end

    Logger.debug("Creating CastingNew Lua systems...")
    local script_attachment = ScriptAttachment.new_static_script(script_asset)
    local TimeResource = world.get_type_by_name("Time<()>")

    -- Create casting system (processes active casts)
    local casting_system = system_builder("CASTING_SYSTEM", script_attachment)
        :resource(TimeResource)
        :query(world.query():component(types.LuaCasting))
        :exclusive()

    local added_casting_system = world.add_system(update_schedule, casting_system)

    -- Create pending skill system (processes pending skills, runs after casting system)
    -- This allows it to process queued skills immediately after a cast finishes
    local pending_system = system_builder("PENDING_SKILL_SYSTEM", script_attachment)
        :query(world.query():component(types.LuaPendingSkill))
        :after(added_casting_system)
        :exclusive()

    world.add_system(update_schedule, pending_system)

    -- Create abnormal effects system
    local abnormal_system = system_builder("ABNORMAL_EFFECTS_SYSTEM", script_attachment)
        :query(world.query():component(types.AbnormalEffects))
        :after(added_casting_system)
        :exclusive()

    world.add_system(update_schedule, abnormal_system)

    local skill_lists_update_system = system_builder("SKILL_LIST_UPDATES", script_attachment)
        :query(world.query()
            :component(types.PaperDoll)
            :component(types.SkillList)
        )
        :after(added_casting_system)
        :exclusive()

    world.add_system(update_schedule, skill_lists_update_system)
end

return Casting
