---@class Skills
local Skills = {}

-- Constructs a Skill object from Lua representation
---@param skill_lua_repr table Lua representation of skill data
---@param skills_storage table The skills storage
---@return Skill The constructed skill object
function Skills.construct(skill_lua_repr, skills_storage)
    local skill_id_type = world.get_type_by_name("game_core::skills::id::Id")
    local skill_id = construct(skill_id_type, { _1 = skill_lua_repr.skill_id })
    local skill_level_type = world.get_type_by_name("game_core::skills::level::Level")
    local skill_level = construct(skill_level_type, { _1 = skill_lua_repr.skill_level })

    -- Get skill handler from storage to extract definition data
    local handler = skills_storage.get(skill_lua_repr.skill_id)
    local definition = handler and handler.definition

    -- Extract magic_level from definition tables
    local magic_level = 1
    if definition and definition.tables and definition.tables.magicLevel then
        local level_index = skill_lua_repr.skill_level
        magic_level = definition.tables.magicLevel[level_index] or
            definition.tables.magicLevel[#definition.tables.magicLevel] or 1
    end

    -- Extract kind from definition
    local skill_kind = (definition and definition.kind) or "Active"

    -- Extract display_id from definition (optional)
    local display_id = nil
    if definition and definition.display_id then
        display_id = construct(skill_id_type, { _1 = definition.display_id })
    end

    local skill_kind_type = world.get_type_by_name("game_core::skills::kind::Kind")

    -- Build skill construction table
    local skill_construction = {
        id = skill_id,
        level = skill_level,
        magic_level = magic_level,
        kind = construct(skill_kind_type, { variant = skill_kind }),
        disabled = skill_lua_repr.disabled or false,
    }

    if display_id then
        skill_construction.display_id = display_id
    end

    ---@type Skill
    local skill_ref = construct(types.Skill, skill_construction)
    return skill_ref
end

return Skills
