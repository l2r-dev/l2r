-- app.lua - Main entry point using the LuaPlugin system
-- Provides clean plugin-based organization for script developers
require("data.scripts.Utils")
require("data.scripts.runtime.skills.types")
local LuaApp = req("data.scripts.LuaApp")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("SkillsApp")


local SkillsPlugin = req("data.scripts.runtime.skills.plugins.SkillsPlugin")

local app = LuaApp:new("SkillsApp")
app:add_plugins({ SkillsPlugin })
app:initialize()


function on_packet_received(entity, packet)
    local skills_plugin = app:get_plugin("SkillsPlugin")
    if skills_plugin then
        ---@cast skills_plugin SkillsPlugin
        skills_plugin:handle_packet(entity, packet)
    end
end

function on_script_loaded()
    Logger.info("Loaded plugins:")
    for _, plugin in pairs(app.plugins) do
        Logger.info("- " .. plugin:to_string())
    end
end
