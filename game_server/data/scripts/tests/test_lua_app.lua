require("data.scripts.TestUtils")
local LuaApp = require("data.scripts.LuaApp")
local LuaPlugin = require("data.scripts.LuaPlugin")

local function run_tests()
    reset_test_stats()

    describe("LuaApp Tests", function()
        it("should create app correctly", function()
            local app = LuaApp:new("TestApp")
            assert_not_nil(app, "App creation")
            assert_eq(app.app_name, "TestApp", "App name")
            local stats = app:get_stats()
            assert_eq(stats.plugins_count, 0, "Initial plugin count")
        end)

        it("should add plugins correctly", function()
            local app = LuaApp:new("TestApp")
            local plugin = LuaPlugin:new("TestPlugin1")
            function plugin:build(app)
                -- Plugin can store its own services
                ---@diagnostic disable-next-line: inject-field
                self.test_service = "test_value"
            end

            app:add_plugins(plugin)
            assert_true(app:has_plugin("TestPlugin1"), "Plugin added successfully")
            local stats = app:get_stats()
            assert_eq(stats.plugins_count, 1, "Plugin count after addition")
        end)

        it("should retrieve plugins correctly", function()
            local app = LuaApp:new("TestApp")
            local plugin = LuaPlugin:new("TestPlugin1")
            function plugin:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.test_service = "test_value"
            end

            app:add_plugins(plugin)
            local retrieved_plugin = app:get_plugin("TestPlugin1")
            assert_not_nil(retrieved_plugin, "Plugin retrieval")
            ---@diagnostic disable-next-line: need-check-nil
            assert_eq(retrieved_plugin.name, "TestPlugin1", "Retrieved plugin name")
            ---@diagnostic disable-next-line: need-check-nil
            assert_eq(retrieved_plugin.test_service, "test_value", "Plugin service accessible")
        end)

        it("should handle app initialization correctly", function()
            local app = LuaApp:new("TestApp")
            local plugin = LuaPlugin:new("TestPlugin1")
            local initialized = false

            function plugin:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.test_service = "test_value"
            end

            function plugin:on_initialize(app)
                initialized = true
            end

            app:add_plugins(plugin)
            app:initialize()
            assert_true(app._initialized, "App initialized")
            assert_true(initialized, "Plugin initialization")
        end)

        it("should enforce plugin uniqueness", function()
            local app = LuaApp:new("UniquenessTestApp")
            local plugin1 = LuaPlugin:new("DuplicateName")
            local plugin2 = LuaPlugin:new("DuplicateName")

            function plugin1:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service1 = "value1"
            end

            function plugin2:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service2 = "value2"
            end

            app:add_plugins(plugin1)

            local success, error_msg = pcall(function()
                app:add_plugins(plugin2)
            end)

            assert_false(success, "Should not allow duplicate plugin names")
            assert_contains(error_msg or "", "already exists", "Error should mention plugin already exists")
        end)

        it("should enforce subplugin uniqueness across app", function()
            local app = LuaApp:new("SubpluginUniquenessTest")

            -- Create parent plugins with children having same names
            local parent1 = LuaPlugin:new("Parent1")
            local parent2 = LuaPlugin:new("Parent2")
            local child1 = LuaPlugin:new("SameChildName")
            local child2 = LuaPlugin:new("SameChildName")

            function child1:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service1 = "value1"
            end

            function child2:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service2 = "value2"
            end

            parent1:add_plugins(child1)
            parent2:add_plugins(child2)

            app:add_plugins(parent1)

            local success, error_msg = pcall(function()
                app:add_plugins(parent2)
            end)

            assert_false(success, "Should detect duplicate subplugin names")
            assert_contains(error_msg or "", "already exists", "Error should mention duplicate name")
        end)

        it("should handle plugin groups correctly", function()
            local app = LuaApp:new("GroupTestApp")

            -- Create child plugins that build themselves
            local child1 = LuaPlugin:new("GroupChild1")
            local child2 = LuaPlugin:new("GroupChild2")

            function child1:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service1 = "value1"
            end

            function child2:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service2 = "value2"
            end

            -- Add the children directly to the app to test individual plugin management
            app:add_plugins(child1)
            app:add_plugins(child2)

            assert_true(app:has_plugin("GroupChild1"), "Group child 1 added")
            assert_true(app:has_plugin("GroupChild2"), "Group child 2 added")
        end)

        it("should detect duplicates in plugin arrays", function()
            local app = LuaApp:new("ArrayDuplicateTest")
            local plugin1 = LuaPlugin:new("ArrayPlugin")
            local plugin2 = LuaPlugin:new("ArrayPlugin")

            function plugin1:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service1 = "value1"
            end

            function plugin2:build(app)
                ---@diagnostic disable-next-line: inject-field
                self.service2 = "value2"
            end

            local success, error_msg = pcall(function()
                app:add_plugins({ plugin1, plugin2 })
            end)

            assert_false(success, "Should detect duplicates in plugin array")
            assert_contains(error_msg or "", "already loaded", "Error should mention already loaded")
        end)
    end)

    -- Return success if all tests passed
    return print_test_summary()
end

-- Execute the tests and return the result
return run_tests()
