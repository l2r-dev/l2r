-- LuaApp.lua - Plugin management system inspired by Bevy's App and Plugin architecture
--
-- Provides a structured way to organize game logic into modular, reusable plugins.
-- Handles plugin registration, dependency resolution, and lifecycle management.
-- Plugins can contain sub-plugins and are loaded with proper dependency ordering.

require("data.scripts.Utils")
require("data.scripts.types")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("LuaApp")

---@class LuaApp
local LuaApp = {}
LuaApp.__index = LuaApp

-- Creates the main application container for the plugin system
---@param name string?
---@return LuaApp
function LuaApp:new(name)
    local app = {
        app_name = name or "LuaApp",
        plugins = {}, -- Stores plugins by name as hashmap
        _initialized = false
    }
    setmetatable(app, self)
    Logger.debug("Created new " .. app.app_name .. " instance")
    return app
end

-- Registers plugins with the application, handling various input formats
-- Plugins are processed in registration order, but actual load order respects dependencies.
-- Plugin priority values may be used for fine-grained ordering within dependency levels.
---@param plugins LuaPlugin|LuaPlugin[]
---@return LuaApp
function LuaApp:add_plugins(plugins)
    if not plugins then
        error("add_plugins() called with nil plugins")
    end

    -- Handle plugin array
    if type(plugins) == "table" and plugins.build and not plugins.name then
        local group_plugins = plugins:build(self)
        if not group_plugins then
            error("Plugin group build() returned nil")
        end
        self:check_plugins_uniqueness(group_plugins)
        for _, plugin in ipairs(group_plugins) do
            self:add_plugin(plugin)
        end
        -- Handle single plugin
    elseif type(plugins) == "table" and plugins.name and plugins.build then
        ---@cast plugins LuaPlugin
        self:check_plugins_uniqueness({ plugins })
        self:add_plugin(plugins)
        -- Handle array of plugins
    elseif type(plugins) == "table" then
        ---@cast plugins LuaPlugin[]
        self:check_plugins_uniqueness(plugins)
        for _, plugin in ipairs(plugins) do
            if type(plugin) == "table" and plugin.name and plugin.build then
                self:add_plugin(plugin)
            else
                error("Invalid plugin in plugins array: " .. tostring(plugin))
            end
        end
    else
        error("Invalid plugins parameter. Expected plugin, plugin group, or array of plugins")
    end

    return self
end

-- Validates plugin name uniqueness across the entire plugin hierarchy
-- Prevents conflicts that would cause module overwrites or ambiguous references
---@param plugins LuaPlugin[]
function LuaApp:check_plugins_uniqueness(plugins)
    local existing_names = self:get_all_plugin_names()
    local new_names = {}

    -- Collect all names from new plugins (including subplugins)
    for _, plugin in ipairs(plugins) do
        self:collect_plugin_names(plugin, new_names)
    end

    -- Check for conflicts with existing plugins
    for name, _ in pairs(new_names) do
        if existing_names[name] then
            error("Plugin name '" .. name .. "' already exists in the application")
        end
    end

    -- Check for conflicts within new plugins
    local name_count = {}
    for name, _ in pairs(new_names) do
        name_count[name] = (name_count[name] or 0) + 1
        if name_count[name] > 1 then
            error("Duplicate plugin name '" .. name .. "' found in plugins being added")
        end
    end
end

-- Collect all plugin names recursively from a plugin
---@param plugin LuaPlugin
---@param names table<PluginName, boolean>
function LuaApp:collect_plugin_names(plugin, names)
    names[plugin.name] = true

    if plugin.plugins and #plugin.plugins > 0 then
        for _, subplugin in ipairs(plugin.plugins) do
            self:collect_plugin_names(subplugin, names)
        end
    end
end

-- Internal method to add a single plugin to the application
---@param plugin table
---@return table
---@private
function LuaApp:add_plugin(plugin)
    if not plugin or not plugin.name or not plugin.build then
        error("Invalid plugin. Expected plugin with 'name' and 'build' methods")
    end

    -- Check if plugin is already loaded
    if self.plugins[plugin.name] then
        error("Plugin '" .. plugin.name .. "' is already loaded, cannot add duplicate")
    end

    -- Check for subplugin uniqueness
    self:check_plugins_uniqueness({ plugin })

    Logger.info("Adding plugin: " .. plugin.name)
    self:check_dependencies(plugin)
    plugin:build(self)

    -- Store plugin reference by name
    self.plugins[plugin.name] = plugin

    -- Mark as initialized and call lifecycle
    plugin._initialized = true
    plugin:on_load(self)
    return self
end

-- Validates dependency resolution before plugin activation
-- Ensures all required plugins are already loaded to prevent runtime failures
---@param plugin table
---@return nil
function LuaApp:check_dependencies(plugin)
    for _, dep_name in ipairs(plugin.dependencies) do
        if not self.plugins[dep_name] then
            error("Plugin '" .. plugin.name .. "' depends on '" .. dep_name .. "' which is not loaded")
        end
    end
end

-- Triggers the initialization phase for all registered plugins
-- Called after all plugins are loaded to perform cross-plugin setup
---@return nil
function LuaApp:initialize()
    if self._initialized then
        Logger.warn("Application already initialized, skipping")
        return
    end

    -- Call any plugin initialization that needs to happen
    for _, plugin in pairs(self.plugins) do
        if plugin.on_initialize then
            plugin:on_initialize(self)
        end
    end

    self._initialized = true
end

---@param name string|nil
---@return table
function LuaApp:set_app_name(name)
    local old_name = self.app_name
    self.app_name = name or "LuaApp"
    Logger.info("Changed app name from '" .. old_name .. "' to '" .. self.app_name .. "'")
    return self
end

---@return string
function LuaApp:get_app_name()
    return self.app_name
end

---@param name string
---@return table|nil
function LuaApp:get_plugin(name)
    return self.plugins[name]
end

---@param name string
---@return boolean
function LuaApp:has_plugin(name)
    return self.plugins[name] ~= nil
end

-- Provides runtime metrics for debugging and monitoring plugin system health
---@return table
function LuaApp:get_stats()
    local plugin_count = 0
    for _ in pairs(self.plugins) do
        plugin_count = plugin_count + 1
    end

    return {
        plugins_count = plugin_count,
        initialized = self._initialized
    }
end

-- Flattens the plugin hierarchy to detect naming conflicts
-- Used during plugin registration to ensure global uniqueness
---@return table
function LuaApp:get_all_plugin_names()
    local all_names = {}

    -- Add direct plugins
    for name, _ in pairs(self.plugins) do
        all_names[name] = true
    end

    -- Add subplugins recursively
    for _, plugin in pairs(self.plugins) do
        local subnames = self:get_plugin_subnames(plugin)
        for subname, _ in pairs(subnames) do
            all_names[subname] = true
        end
    end

    return all_names
end

-- Get all subplugin names for a given plugin
---@param plugin table
---@return table
function LuaApp:get_plugin_subnames(plugin)
    local subnames = {}

    if plugin.plugins and #plugin.plugins > 0 then
        for _, subplugin in ipairs(plugin.plugins) do
            subnames[subplugin.name] = true
            -- Recursively get subplugin names
            local nested_subnames = self:get_plugin_subnames(subplugin)
            for nested_name, _ in pairs(nested_subnames) do
                subnames[nested_name] = true
            end
        end
    end

    return subnames
end

---@return string
function LuaApp:to_string()
    local stats = self:get_stats()
    return string.format("%s{ plugins=%d, initialized=%s }",
        self.app_name,
        stats.plugins_count,
        tostring(stats.initialized)
    )
end

return LuaApp
