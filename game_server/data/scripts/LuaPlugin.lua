-- LuaPlugin.lua - Modular plugin system inspired by Bevy's Plugins
--
-- Plugins encapsulate game logic into reusable, composable units. Each plugin can:
-- - Expose services and functionality directly through their interface
-- - Declare dependencies on other plugins for load ordering
-- - Act as a container for child plugins (plugin groups)
-- - Define lifecycle hooks for initialization and cleanup
-- - Use priority values for fine-grained load ordering within dependency levels

require("data.scripts.Utils")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("LuaPlugin")


---@alias PluginName string The unique name identifier for a plugin
---@alias PluginRegistry table<PluginName, LuaPlugin> Maps plugin names to plugin instances

---@class PluginConfig
---@field [string] any Key-value configuration table for plugins

---@class LuaPlugin
---@field name PluginName The unique name of the plugin
---@field config PluginConfig Configuration table for the plugin
---@field dependencies PluginName[] List of plugin names this plugin depends on
---@field enabled boolean Whether the plugin is currently enabled
---@field priority number Priority for load ordering within dependency levels (default: 100)
---@field _initialized boolean Internal flag indicating if plugin has been initialized
---@field plugins LuaPlugin[] Array of child plugins
---@field disabled_plugins table<PluginName, boolean> Map of disabled child plugin names
---@field configured_plugins table<PluginName, PluginConfig> Map of child plugin configurations
---@field _built boolean Internal flag indicating if plugin container has been built
local LuaPlugin = {}
LuaPlugin.__index = LuaPlugin

-- Creates a new plugin instance with configurable behavior
---@param name PluginName?
---@param config PluginConfig?
---@return LuaPlugin
function LuaPlugin:new(name, config)
    local plugin = {
        name = name or "UnnamedPlugin",
        config = config or {},
        app = nil,
        dependencies = {},
        enabled = true,
        priority = 100,
        _initialized = false,
        plugins = {},
        disabled_plugins = {},
        configured_plugins = {},
        _built = false,
    }
    setmetatable(plugin, self)
    return plugin
end

-- Core plugin interface method, equivalent to Bevy's Plugin::build()
-- When implemented by concrete plugins: registers modules and sets up plugin state
-- When used as a plugin container: processes and returns child plugins with configuration applied
---@param app LuaApp
---@return LuaPlugin[]?
function LuaPlugin:build(app)
    -- Store app reference for child plugin building
    self.app = app

    -- If this plugin has children, act as a container
    if #self.plugins > 0 then
        return self:build_children(app)
    else
        -- Otherwise, this is the original build method for plugin implementation
        error("Plugin '" .. self.name .. "' must implement build(app) method")
    end
end

-- Plugin lifecycle hooks - override these to customize plugin behavior
-- Called at specific points during application startup and shutdown

-- Called immediately after plugin registration and module setup
---@param app LuaApp
function LuaPlugin:on_load(app)
    Logger.trace("Plugin '" .. self.name .. "' loaded")
end

-- Called when plugin is being removed from the application
---@param app LuaApp
function LuaPlugin:on_unload(app)
    Logger.trace("Plugin '" .. self.name .. "' unloaded")
end

-- Called when plugin transitions from disabled to enabled state
---@param app LuaApp
function LuaPlugin:on_enable(app)
    Logger.trace("Plugin '" .. self.name .. "' enabled")
end

-- Called when plugin transitions from enabled to disabled state
---@param app LuaApp
function LuaPlugin:on_disable(app)
    Logger.trace("Plugin '" .. self.name .. "' disabled")
end

-- Called during app.initialize() after all plugins are loaded
-- Use this for cross-plugin setup that requires all dependencies to be present
---@param app LuaApp
function LuaPlugin:on_initialize(app)
    Logger.trace("Plugin '" .. self.name .. "' initialized")
end

-- Declares a dependency on another plugin for load ordering
-- Dependencies are loaded before this plugin, ensuring required services are available
-- Priority values can provide additional ordering within the same dependency level
---@param plugin_name PluginName
---@return LuaPlugin
function LuaPlugin:add_dependency(plugin_name)
    table.insert(self.dependencies, plugin_name)
    return self
end

---@return boolean
function LuaPlugin:is_enabled()
    return self.enabled
end

---@param app LuaApp
function LuaPlugin:enable(app)
    if not self.enabled then
        self.enabled = true
        self:on_enable(app)
        Logger.info("Plugin '" .. self.name .. "' enabled")
    end
end

-- Disables this plugin or a child plugin (when used as container)
-- Handles both self-disabling and child plugin management
---@param app_or_plugin_name LuaApp|PluginName
---@return LuaPlugin?
function LuaPlugin:disable(app_or_plugin_name)
    -- If it's a string, treat as child plugin disable
    if type(app_or_plugin_name) == "string" then
        return self:disable_child(app_or_plugin_name)
    else
        -- Otherwise, disable this plugin itself (original behavior)
        local app = app_or_plugin_name
        if self.enabled then
            self.enabled = false
            self:on_disable(app)
            Logger.info("Plugin '" .. self.name .. "' disabled")
        end
    end
end

---@param key string
---@param default any
---@return any
function LuaPlugin:get_config(key, default)
    return self.config[key] or default
end

---@param key string
---@param value any
function LuaPlugin:set_config(key, value)
    self.config[key] = value
    Logger.debug("Set config '" .. key .. "' to '" .. tostring(value) .. "' for plugin '" .. self.name .. "'")
end

---@return string
function LuaPlugin:to_string()
    return string.format("{ name='%s', enabled=%s }",
        self.name,
        tostring(self.enabled)
    )
end

-- Plugin container functionality - allows plugins to group and configure other plugins
-- Similar to Bevy's plugin groups, this enables hierarchical plugin organization

-- Adds multiple child plugins when this plugin acts as a container/group
-- Handles single plugins, arrays of plugins, and nested plugin containers
---@param plugins LuaPlugin|LuaPlugin[]
---@return LuaPlugin
function LuaPlugin:add_plugins(plugins)
    if not plugins then
        error("add_plugins() called with nil plugins")
    end

    -- Handle single plugin
    if type(plugins) == "table" and plugins.name then
        ---@cast plugins LuaPlugin
        self:add_plugin(plugins)
        -- Handle array of plugins
    elseif type(plugins) == "table" then
        ---@cast plugins LuaPlugin[]
        for _, plugin in ipairs(plugins) do
            if type(plugin) == "table" and plugin.name then
                self:add_plugin(plugin)
            else
                error("Invalid plugin in plugins array: " .. tostring(plugin))
            end
        end
    else
        error("Invalid plugins parameter. Expected plugin or array of plugins")
    end

    return self
end

-- Internal method to add a single plugin with proper child building
-- Handles both individual plugins and nested plugin containers
---@param plugin LuaPlugin
---@return LuaPlugin
---@private
function LuaPlugin:add_plugin(plugin)
    if type(plugin) ~= "table" or not plugin.name then
        error("Invalid plugin passed to add_single_plugin(). Expected plugin with 'name' field.")
    end

    -- Check if this plugin has children (and a build method)
    if #plugin.plugins > 0 and plugin.build then
        -- This plugin has children, so build it and add all its children
        -- We need an app reference to build children properly
        if self.app then
            local child_plugins = plugin:build_children(self.app)
            if child_plugins then
                for _, child_plugin in ipairs(child_plugins) do
                    self:check_child_plugin_uniqueness(child_plugin)
                    table.insert(self.plugins, child_plugin)
                end
            end
        else
            -- If no app reference yet, just add the plugin and it will be built later
            self:check_child_plugin_uniqueness(plugin)
            table.insert(self.plugins, plugin)
        end
        Logger.debug("Added plugin with children '" .. plugin.name .. "' into '" .. self.name .. "'")
    else
        -- This is a regular plugin, add it directly
        self:check_child_plugin_uniqueness(plugin)
        table.insert(self.plugins, plugin)
        Logger.debug("Added plugin '" .. plugin.name .. "' into '" .. self.name .. "'")
    end

    return self
end

-- Disable a child plugin by name
-- @param plugin_name string - Name of the plugin to disable
-- @return self for method chaining
---@param plugin_name PluginName
---@return LuaPlugin
function LuaPlugin:disable_child(plugin_name)
    self.disabled_plugins[plugin_name] = true
    Logger.debug("Disabled plugin '" .. plugin_name .. "' in '" .. self.name .. "'")
    return self
end

-- Configure a child plugin
-- @param plugin_name string - Name of the plugin to configure
-- @param config table - Configuration to apply to the plugin
-- @return self for method chaining
---@param plugin_name PluginName
---@param config PluginConfig
---@return LuaPlugin
function LuaPlugin:set(plugin_name, config)
    if type(config) ~= "table" then
        error("Plugin configuration must be a table")
    end
    self.configured_plugins[plugin_name] = config
    Logger.debug("Configured plugin '" .. plugin_name .. "' in '" .. self.name .. "'")
    return self
end

-- Processes child plugins with configuration and filtering applied
-- Returns only enabled plugins with their configurations merged
-- Maintains insertion order while respecting disable flags
-- Calls build(app) on each child plugin to ensure proper initialization
---@param app LuaApp
---@return LuaPlugin[]
function LuaPlugin:build_children(app)
    -- If this plugin has children, process them
    if #self.plugins > 0 then
        local enabled_plugins = {}
        local seen_names = {} -- Track plugin names to ensure uniqueness

        -- Process plugins in the order they were added
        for _, plugin in ipairs(self.plugins) do
            if not self.disabled_plugins[plugin.name] then
                -- Check for duplicate names in the result set
                if seen_names[plugin.name] then
                    error("Duplicate plugin name '" ..
                        plugin.name .. "' found while building children for '" .. self.name .. "'")
                end
                seen_names[plugin.name] = true

                -- Apply configuration directly to the plugin
                if self.configured_plugins[plugin.name] then
                    for k, v in pairs(self.configured_plugins[plugin.name]) do
                        plugin.config[k] = v
                    end
                    Logger.debug("Applied configuration to plugin '" .. plugin.name .. "' in '" .. self.name .. "'")
                end

                -- If the child plugin has its own children, build them first
                if #plugin.plugins > 0 then
                    local child_plugins = plugin:build(app)
                    if child_plugins then
                        for _, child_plugin in ipairs(child_plugins) do
                            table.insert(enabled_plugins, child_plugin)
                            Logger.debug("Built child plugin '" .. child_plugin.name .. "' for '" .. self.name .. "'")
                        end
                    end
                else
                    -- This is a leaf plugin, add it directly
                    table.insert(enabled_plugins, plugin)
                    Logger.debug("Built plugin '" .. plugin.name .. "' for '" .. self.name .. "'")
                end
            else
                Logger.debug("Skipped disabled plugin '" .. plugin.name .. "' in '" .. self.name .. "'")
            end
        end

        self._built = true
        Logger.info("Built plugin '" .. self.name .. "' with " .. #enabled_plugins .. " children")
        return enabled_plugins
    else
        -- This is a regular plugin, just return itself if enabled
        if self.enabled then
            return { self }
        else
            return {}
        end
    end
end

-- Check if a child plugin name is unique within this plugin
-- @param plugin LuaPlugin - The plugin to check
---@param plugin LuaPlugin
function LuaPlugin:check_child_plugin_uniqueness(plugin)
    for _, existing_plugin in ipairs(self.plugins) do
        if existing_plugin.name == plugin.name then
            error("Child plugin name '" .. plugin.name .. "' already exists in plugin '" .. self.name .. "'")
        end
    end
end

-- Get a list of all child plugin names
-- @return table - Array of plugin names
---@return PluginName[]
function LuaPlugin:get_children_names()
    local names = {}
    for _, plugin in ipairs(self.plugins) do
        table.insert(names, plugin.name)
    end
    return names
end

return LuaPlugin
