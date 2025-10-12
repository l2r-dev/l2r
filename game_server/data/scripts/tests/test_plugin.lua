require("data.scripts.TestUtils")
local LuaPlugin = require("data.scripts.LuaPlugin")

local function run_tests()
    reset_test_stats()

    describe("LuaPlugin Tests", function()
        local plugin, child_plugin1, child_plugin2

        it("should create plugin correctly", function()
            plugin = LuaPlugin:new("TestPlugin", { setting1 = "value1" })
            assert_not_nil(plugin, "Plugin creation")
            assert_eq(plugin.name, "TestPlugin", "Plugin name")
            assert_true(plugin.enabled, "Plugin enabled by default")
            assert_eq(plugin.priority, 100, "Default priority")
        end)

        it("should handle configuration correctly", function()
            plugin:set_config("setting2", "value2")
            assert_eq(plugin:get_config("setting1"), "value1", "Get existing config")
            assert_eq(plugin:get_config("setting2"), "value2", "Get new config")
            assert_eq(plugin:get_config("nonexistent", "default"), "default", "Default value for missing config")
        end)

        it("should manage dependencies correctly", function()
            plugin:add_dependency("DatabasePlugin")
            plugin:add_dependency("LoggerPlugin")
            assert_length(plugin.dependencies, 2, "Dependency count")
            assert_eq(plugin.dependencies[1], "DatabasePlugin", "First dependency")
            assert_eq(plugin.dependencies[2], "LoggerPlugin", "Second dependency")
        end)

        it("should add individual plugins correctly", function()
            child_plugin1 = LuaPlugin:new("ChildPlugin1")
            child_plugin2 = LuaPlugin:new("ChildPlugin2")

            plugin:add_plugins(child_plugin1)
            plugin:add_plugins(child_plugin2)

            assert_length(plugin.plugins, 2, "Child plugin count")
            assert_eq(plugin.plugins[1].name, "ChildPlugin1", "First child plugin name")
            assert_eq(plugin.plugins[2].name, "ChildPlugin2", "Second child plugin name")
        end)

        it("should add plugin groups correctly", function()
            local child_plugin_container = LuaPlugin:new("ChildPluginContainer")
            local child1 = LuaPlugin:new("Child1")
            local child2 = LuaPlugin:new("Child2")

            child_plugin_container:add_plugins(child1)
            child_plugin_container:add_plugins(child2)

            local main_plugin = LuaPlugin:new("MainPlugin")
            main_plugin:add_plugins(child_plugin_container)

            -- The container should be added as-is, not flattened at this level
            assert_eq(#main_plugin.plugins, 1, "Main plugin should have the container")
            assert_eq(main_plugin.plugins[1].name, "ChildPluginContainer", "Container plugin name")
        end)

        it("should get child plugin names correctly", function()
            local names = plugin:get_children_names()
            assert_length(names, 2, "Plugin names count")
            assert_eq(names[1], "ChildPlugin1", "First plugin name")
            assert_eq(names[2], "ChildPlugin2", "Second plugin name")
        end)

        it("should disable child plugins correctly", function()
            plugin:disable("ChildPlugin1")
            assert_true(plugin.disabled_plugins["ChildPlugin1"], "Child plugin disabled")
            assert_nil(plugin.disabled_plugins["ChildPlugin2"], "Other child plugin not affected")
        end)

        it("should configure child plugins correctly", function()
            plugin:set("ChildPlugin2", { custom_setting = "test_value" })
            assert_not_nil(plugin.configured_plugins["ChildPlugin2"], "Child plugin configuration exists")
            assert_eq(plugin.configured_plugins["ChildPlugin2"].custom_setting, "test_value", "Configuration value set")
        end)

        it("should enforce child plugin uniqueness", function()
            local parent = LuaPlugin:new("UniqueChildTest")
            local child1 = LuaPlugin:new("SameName")
            local child2 = LuaPlugin:new("SameName")

            parent:add_plugins(child1)

            local success, error_msg = pcall(function()
                parent:add_plugins(child2)
            end)

            assert_false(success, "Should not allow duplicate child plugin names")
            assert_contains(error_msg or "", "already exists", "Error message should mention duplicate")
        end)

        it("should enforce uniqueness during build_children", function()
            local parent = LuaPlugin:new("BuildUniquenessTest")
            local app = { name = "TestApp" } -- Mock app for build_children

            -- Create plugins with same name but bypass add_plugins uniqueness by adding directly to plugins array
            local child1 = LuaPlugin:new("DuplicateName")
            local child2 = LuaPlugin:new("DuplicateName")

            -- Directly insert into plugins array to bypass the add_plugins uniqueness check
            table.insert(parent.plugins, child1)
            table.insert(parent.plugins, child2)

            local success, error_msg = pcall(function()
                parent:build_children(app)
            end)

            assert_false(success, "Should detect duplicates during build")
            assert_contains(error_msg or "", "Duplicate plugin name", "Error should mention duplicate name")
        end)
    end)

    return print_test_summary()
end

-- Execute the tests and return the result
return run_tests()
