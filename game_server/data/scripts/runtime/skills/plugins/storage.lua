---Skills Storage Module
---Provides encapsulated storage for skill definitions with a clean API

require("data.scripts.Utils")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("skills.storage")

---@class SkillsStorage
local SkillsStorage = {}

---@type table<number, SkillHandler>
local storage = {}

---Validates that a skill ID is a number
---@param skill_id number The skill ID to validate
local function validate_skill_id(skill_id)
    if type(skill_id) ~= "number" then
        error("Skill ID must be a number, got " .. type(skill_id))
    end
end

---Validates that a skill handler has required structure and functions
---@param handler SkillHandler The skill handler to validate
local function validate_handler(handler)
    if type(handler) ~= "table" then
        error("Skill handler must be a table, got " .. type(handler))
    end

    -- Validate required definition field
    if not handler.definition then
        error("Skill handler must have a 'definition' field")
    end
    if type(handler.definition) ~= "table" then
        error("Skill handler 'definition' must be a table, got " .. type(handler.definition))
    end

    -- Validate definition has required fields
    local def = handler.definition
    if type(def.id) ~= "number" then
        error("Skill definition must have a numeric 'id' field")
    end
    if type(def.kind) ~= "string" then
        error("Skill definition must have a string 'kind' field (Active/Passive/Toggle)")
    end

    -- Validate functions based on skill kind
    if def.kind == "Active" or def.kind == "Toggle" then
        if type(handler.pend) ~= "function" then
            error(def.kind .. " skill handler must have a 'pend' function")
        end
        if type(handler.on_pending) ~= "function" then
            error(def.kind .. " skill handler must have an 'on_pending' function")
        end
        if type(handler.launch) ~= "function" then
            error(def.kind .. " skill handler must have a 'launch' function")
        end
        -- Toggle skills always need apply_abnormal
        if def.kind == "Toggle" and type(handler.apply_abnormal) ~= "function" then
            error("Toggle skill handler must have an 'apply_abnormal' function")
        end
    elseif def.kind == "Passive" then
        if type(handler.apply_passive) ~= "function" then
            error("Passive skill handler must have an 'apply_passive' function")
        end
    else
        Logger.warn("Unknown skill kind '" .. tostring(def.kind) .. "' for skill " .. tostring(def.id))
    end

    -- Optional: validate apply_abnormal for buff/debuff skills
    if handler.apply_abnormal and type(handler.apply_abnormal) ~= "function" then
        error("Skill handler 'apply_abnormal' must be a function if present")
    end
end

---Sets a skill definition in storage
---@param skill_id number The skill ID
---@param handler SkillHandler The skill definition
function SkillsStorage.set(skill_id, handler)
    validate_skill_id(skill_id)
    validate_handler(handler)

    local old_skill = storage[skill_id]
    storage[skill_id] = handler

    Logger.trace("Skill " .. tostring(skill_id) .. " " .. (old_skill and "updated" or "added") .. " in storage")
end

---Gets a skill definition from storage, loading it if necessary
---@param skill_id number The skill ID
---@return SkillHandler|nil skill The skill definition or nil if not found
function SkillsStorage.get(skill_id)
    -- Return skill if already loaded
    local skill = storage[skill_id]
    if skill then
        return skill
    end

    local module_path = "data.scripts.runtime.skills.definitions." .. tostring(skill_id)
    local success, error_msg = pcall(function()
        SkillsStorage._load_from_file(skill_id, module_path)
    end)

    if success then
        return storage[skill_id]
    else
        Logger.debug("Failed to load skill file " .. tostring(skill_id) .. ".lua")
        return nil
    end
end

---Loads a skill definition from a file
---@param skill_id number The skill ID
---@param file_path string The path to the skill definition file (relative to scripts root)
---@return boolean success Whether the skill was loaded successfully
---@private
function SkillsStorage._load_from_file(skill_id, file_path)
    validate_skill_id(skill_id)

    local success, handler = pcall(req, file_path)
    if not success then
        Logger.error("Failed to load skill " .. tostring(skill_id) .. " from " .. file_path)
        return false
    end
    SkillsStorage.set(skill_id, handler)
    return true
end

---Counts the number of loaded skills in storage
---@return number count The number of skills currently in storage
function SkillsStorage.count()
    local count = 0
    for _ in pairs(storage) do
        count = count + 1
    end
    return count
end

---Checks if a skill exists in storage
---@param skill_id number The skill ID to check
---@return boolean exists Whether the skill exists in storage
function SkillsStorage.has(skill_id)
    return storage[skill_id] ~= nil
end

---Clears all skills from storage
function SkillsStorage.clear()
    storage = {}
    Logger.info("Skills storage cleared")
end

return SkillsStorage
