local LuaPlugin = req("data.scripts.LuaPlugin")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("skills.plugin")

---@class SkillsPlugin : LuaPlugin
---@field storage SkillsStorage Skills storage module
---@field casting SkillCasting Skills casting module
---@field packets SkillPackets Skills packets module
local SkillsPlugin = LuaPlugin:new("SkillsPlugin")

-- Builds the plugin by loading required modules
---@param app LuaApp The application instance
function SkillsPlugin:build(app)
    self.storage = req("data.scripts.runtime.skills.plugins.storage")
    self.casting = req("data.scripts.runtime.skills.plugins.casting")
    self.packets = req("data.scripts.runtime.skills.plugins.packets")
end

-- Called when plugin is loaded into the application
---@param app LuaApp The application instance
function SkillsPlugin:on_load(app)
    if self.casting then
        self.casting.create_components()
        self.casting.create_systems(self.storage)
    end
end

-- Called when plugin is unloaded from the application
---@param app LuaApp The application instance
function SkillsPlugin:on_unload(app)
    Logger.info("SkillsPlugin unloaded")
end

-- Retrieves a skill definition by ID
---@param skill_id number The skill ID to retrieve
---@return table|nil The skill definition or nil if not found
function SkillsPlugin:get_skill(skill_id)
    return self.storage.get(skill_id)
end

-- Checks if a skill exists in storage
---@param skill_id number The skill ID to check
---@return boolean True if skill exists, false otherwise
function SkillsPlugin:has_skill(skill_id)
    return self:get_skill(skill_id) ~= nil
end

-- Handles incoming packets related to skills
---@param entity Entity The entity that sent the packet
---@param packet GameServerPacket The packet to handle
---@return boolean|nil Result of packet handling
function SkillsPlugin:handle_packet(entity, packet)
    if not self.packets then
        Logger.error("SkillsPackets service not available")
        return
    end

    if not self.storage then
        Logger.error("SkillsStorage service not available")
        return
    end

    -- Pass the storage to the packet handler
    self.packets.on_packet_received(entity, packet, self.storage)
end

function SkillsPlugin:get_skill_stats()
    if self.storage then
        return {
            total_skills = self.storage.count(),
            plugin_enabled = self:is_enabled()
        }
    end
    return {
        total_skills = 0,
        plugin_enabled = self:is_enabled(),
        error = "SkillsStorage not available"
    }
end

function SkillsPlugin:get_storage()
    return self.storage
end

return SkillsPlugin
