local Logger = req("data.scripts.Logger")
local Skills = req("data.scripts.game.Skills")
Logger.set_script_name("game.SkillList")

---@class SkillList
local SkillList = {}

-- Global table to store last change ticks for each entity
local last_change_ticks = {}

--[[
    Checks if an entity's SkillList has changed since the last check
    @param entity The entity to check
    @return true if SkillList has changed, false otherwise
]]
function SkillList.is_changed(entity)
    local entity_key = tostring(entity:index())
    local component_ticks = world.get_component_ticks(entity, SkillList.type())

    -- Get the last change tick for this entity, or create a new one if it doesn't exist
    local last_tick = last_change_ticks[entity_key]
    if not last_tick then
        last_tick = construct(types.Tick, { tick = 0 })
        last_change_ticks[entity_key] = last_tick
    end

    -- Check if the SkillList has changed
    local is_changed = component_ticks:is_changed(last_tick, component_ticks.changed)

    -- Update the last change tick for this entity if it has changed
    if is_changed then
        last_change_ticks[entity_key] = component_ticks.changed
    end

    return is_changed
end

function SkillList.update_change_tick(entity)
    local entity_key = tostring(entity:index())
    local component_ticks = world.get_component_ticks(entity, SkillList.type())
    last_change_ticks[entity_key] = component_ticks.changed
end

--- Loads character skills from database
---@param char_id ObjectId The character's ObjectId
---@return table character_skills List of skills from database
function SkillList.load_skills_from_db(char_id)
    local character_skills = DatabaseOps.query_raw({
        sql = "SELECT skill_id, skill_level FROM character_skills WHERE char_id = $1",
        params = { char_id._1 },
        return_multiple = true
    })
    return character_skills
end

--- Syncs current skill list to database with optimal diff-based updates
--- Only updates skills that changed (new, level changed, or deleted)
---@param char_id ObjectId The character's ObjectId
---@param skill_list SkillList The current SkillList component
---@param sub_class_variant SubClassVariant The SubClassVariant enum
---@param skills_storage SkillsStorage The skills storage for lazy loading
---@return number updated_count Number of skills updated
function SkillList.sync_skills_to_db(char_id, skill_list, sub_class_variant, skills_storage)
    -- Load existing skills from database
    local db_skills = SkillList.load_skills_from_db(char_id)

    -- Build a map of database skills for quick lookup
    local db_skills_map = {}
    for _, db_skill in ipairs(db_skills) do
        db_skills_map[db_skill.skill_id] = db_skill.skill_level
    end

    -- Build a map of current skills from the skill list
    local current_skills_map = {}

    for _, skill in pairs(skill_list._1) do
        if skill and skill.id and skill.level then
            current_skills_map[skill.id._1] = skill.level._1
        end
    end

    local updated_count = 0
    local char_skill_model_type = world.get_type_by_name("game_core::character::skills::Model")

    -- Find skills to upsert (new or level changed)
    for skill_id, skill_level in pairs(current_skills_map) do
        local db_level = db_skills_map[skill_id]
        if not db_level or db_level ~= skill_level then
            -- Preload skill handler from storage (lazy loading)
            if skills_storage then
                local handler = skills_storage.get(skill_id)
                if not handler then
                    Logger.warn("Skill handler not found for skill ID " .. tostring(skill_id) .. " during DB sync")
                end
            end

            -- Skill is new or level changed, upsert it
            local skill_id_type = world.get_type_by_name("game_core::skills::id::Id")
            local skill_id_obj = construct(skill_id_type, { _1 = skill_id })

            local char_skill_model = construct(char_skill_model_type, {
                char_id = char_id,
                skill_id = skill_id_obj,
                sub_class = sub_class_variant,
                skill_level = skill_level
            })

            local success = DatabaseOps.create_or_update(char_skill_model)
            if success then
                updated_count = updated_count + 1
            else
                Logger.error("Failed to sync skill " .. tostring(skill_id) .. " to database")
            end
        end
    end

    if updated_count > 0 then
        Logger.debug("Synced " ..
            tostring(updated_count) .. " skill changes to database for character " .. tostring(char_id._1))
    end

    return updated_count
end

--- Saves a skill to the database for a character
---@param char_id ObjectId The character's ObjectId
---@param skill_node SkillTreeNode The skill node from skill tree
---@param sub_class_variant SubClassVariant The SubClassVariant enum
---@return boolean success Whether the save was successful
function SkillList.save_skill_to_db(char_id, skill_node, sub_class_variant)
    local char_skill_model_type = world.get_type_by_name("game_core::character::skills::Model")
    local char_skill_model = construct(char_skill_model_type, {
        char_id = char_id,
        skill_id = skill_node.skill_id,
        sub_class = sub_class_variant,
        skill_level = skill_node.skill_level._1
    })

    local success = DatabaseOps.create_or_update(char_skill_model)
    if not success then
        Logger.error("Failed to save skill " .. tostring(skill_node.skill_id._1) .. " to database")
    end
    return success
end

--- Grants auto-skills to a character and saves them to the database
---@param entity Entity The entity object
---@param char_id ObjectId The character's ObjectId
---@param skill_list SkillList The entity's SkillList component
---@param class_tree_nodes SkillTreeNodes The skill tree nodes from SkillList._1
---@param char_level number The character's current level
---@param sub_class_variant SubClassVariant The SubClassVariant enum
---@param skills_storage table The skills storage
---@return number count Number of skills granted
function SkillList.grant_auto_skills(entity, char_id, skill_list, class_tree_nodes, char_level, sub_class_variant,
                                     skills_storage)
    local auto_skills = SkillList.auto_skill_on_level(class_tree_nodes, char_level)

    for _, skill_node in ipairs(auto_skills) do
        -- Create skill reference from node
        local skill_data = {
            skill_id = skill_node.skill_id._1,
            skill_level = skill_node.skill_level._1
        }
        local skill_ref = Skills.construct(skill_data, skills_storage)

        -- Add to skill list
        skill_list._1[skill_ref.id] = skill_ref

        -- Save to database
        SkillList.save_skill_to_db(char_id, skill_node, sub_class_variant)
    end

    return #auto_skills
end

--- Loads skills from database and adds them to the skill list
---@param skill_list SkillList The entity's SkillList component
---@param character_skills table List of skills from database
---@param skills_storage table The skills storage
function SkillList.load_skills_to_list(skill_list, character_skills, skills_storage)
    for _, skill in ipairs(character_skills) do
        local skill_ref = Skills.construct(skill, skills_storage)
        skill_list._1[skill_ref.id] = skill_ref
    end
end

--- Filters auto-skills for a specific level from skill tree nodes
--- Equivalent to Rust's SkillTree::auto_skill_on_level method
---@param class_tree_nodes SkillTreeNodes The skill tree nodes from SkillList._1
---@param char_level number The character's current level
---@return SkillTreeNode[] auto_skills List of skill nodes that should be auto-granted
function SkillList.auto_skill_on_level(class_tree_nodes, char_level)
    local auto_skills = {}
    local nodes_len = class_tree_nodes:len()
    if not nodes_len then
        Logger.error("Failed to get length of class_tree_nodes")
        return auto_skills
    end

    for i = 1, nodes_len do
        local skill_node = class_tree_nodes[i]
        if skill_node then
            local has_auto, has_level = false, false
            local requirements = skill_node.requirements._1

            if requirements then
                for j = 1, requirements:len() or 0 do
                    local requirement = requirements[j]
                    if requirement then
                        local variant_name = requirement:variant_name()
                        if variant_name == "Auto" then
                            has_auto = true
                        elseif variant_name == "Level" and tonumber(requirement._1._1) == tonumber(char_level) then
                            has_level = true
                        end
                        if has_auto and has_level then break end
                    end
                end
            end

            if has_auto and has_level then
                table.insert(auto_skills, skill_node)
            end
        end
    end

    return auto_skills
end

function SkillList.type()
    return types.SkillList
end

return SkillList
