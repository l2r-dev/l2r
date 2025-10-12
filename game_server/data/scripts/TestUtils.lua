---@diagnostic disable: lowercase-global
-- TestUtils.lua - Global test assertion framework with descriptive output
--
-- Provides a comprehensive set of assertion functions that automatically track
-- test results and generate detailed failure messages. Functions are made global
-- to enable natural test syntax without require() boilerplate.

-- Tracks pass/fail statistics across all tests in the current session
local test_stats = {
    passed = 0,
    failed = 0,
    total = 0
}

-- Converts values to human-readable strings for assertion failure messages
---@param value any The value to format as string
---@return string Human-readable string representation
local function format_value(value)
    if type(value) == "string" then
        return '"' .. value .. '"'
    elseif type(value) == "table" then
        local items = {}
        for k, v in pairs(value) do
            table.insert(items, tostring(k) .. "=" .. format_value(v))
        end
        return "{" .. table.concat(items, ", ") .. "}"
    else
        return tostring(value)
    end
end


---@param passed boolean Whether the test passed
---@param message? string Optional test message
---@param details? string Optional details for failure
---@return boolean Returns passed value
local function print_result(passed, message, details)
    test_stats.total = test_stats.total + 1
    if passed then
        test_stats.passed = test_stats.passed + 1
        print("✓ PASS: " .. (message or "assertion"))
    else
        test_stats.failed = test_stats.failed + 1
        print("✗ FAIL: " .. (message or "assertion") .. (details and (" - " .. details) or ""))
    end
    return passed
end

-- Core assertion functions - automatically track results and provide detailed failure messages

---@param actual any Actual value
---@param expected any Expected value
---@param message? string Optional message
---@return boolean Assertion result
function assert_eq(actual, expected, message)
    local passed = actual == expected
    local details = passed and nil or ("expected " .. format_value(expected) .. ", got " .. format_value(actual))
    return print_result(passed, message, details)
end

---@param actual any Actual value
---@param expected any Expected value
---@param message? string Optional message
---@return boolean Assertion result
function assert_not_eq(actual, expected, message)
    local passed = actual ~= expected
    local details = passed and nil or ("expected not " .. format_value(expected) .. ", but got " .. format_value(actual))
    return print_result(passed, message, details)
end

---@param value any Value to check
---@param message? string Optional message
---@return boolean Assertion result
function assert_true(value, message)
    return assert_eq(value, true, message or "should be true")
end

---@param value any Value to check
---@param message? string Optional message
---@return boolean Assertion result
function assert_false(value, message)
    return assert_eq(value, false, message or "should be false")
end

---@param value any Value to check
---@param message? string Optional message
---@return boolean Assertion result
function assert_not_nil(value, message)
    local passed = value ~= nil
    local details = passed and nil or "value was nil"
    return print_result(passed, message or "should not be nil", details)
end

---@param value any Value to check
---@param message? string Optional message
---@return boolean Assertion result
function assert_nil(value, message)
    local passed = value == nil
    local details = passed and nil or ("expected nil, got " .. format_value(value))
    return print_result(passed, message or "should be nil", details)
end

---@param value any Value to check
---@param expected_type string Expected type
---@param message? string Optional message
---@return boolean Assertion result
function assert_type(value, expected_type, message)
    local actual_type = type(value)
    local passed = actual_type == expected_type
    local details = passed and nil or ("expected type " .. expected_type .. ", got " .. actual_type)
    return print_result(passed, message or ("should be of type " .. expected_type), details)
end

---@param actual number Actual value
---@param expected number Expected value
---@param message? string Optional message
---@return boolean Assertion result
function assert_greater_than(actual, expected, message)
    local passed = actual > expected
    local details = passed and nil or (format_value(actual) .. " is not greater than " .. format_value(expected))
    return print_result(passed, message or "should be greater than", details)
end

---@param actual number Actual value
---@param expected number Expected value
---@param message? string Optional message
---@return boolean Assertion result
function assert_greater_or_equal(actual, expected, message)
    local passed = actual >= expected
    local details = passed and nil or
        (format_value(actual) .. " is not greater than or equal to " .. format_value(expected))
    return print_result(passed, message or "should be greater than or equal", details)
end

---@param actual number Actual value
---@param expected number Expected value
---@param message? string Optional message
---@return boolean Assertion result
function assert_less_than(actual, expected, message)
    local passed = actual < expected
    local details = passed and nil or (format_value(actual) .. " is not less than " .. format_value(expected))
    return print_result(passed, message or "should be less than", details)
end

---@param actual number Actual value
---@param expected number Expected value
---@param message? string Optional message
---@return boolean Assertion result
function assert_less_or_equal(actual, expected, message)
    local passed = actual <= expected
    local details = passed and nil or
        (format_value(actual) .. " is not less than or equal to " .. format_value(expected))
    return print_result(passed, message or "should be less than or equal", details)
end

---@param table_or_string table|string Table or string to search
---@param value any Value to find
---@param message? string Optional message
---@return boolean Assertion result
function assert_contains(table_or_string, value, message)
    local passed = false

    if type(table_or_string) == "table" then
        for _, v in pairs(table_or_string) do
            if v == value then
                passed = true
                break
            end
        end
    elseif type(table_or_string) == "string" and type(value) == "string" then
        passed = string.find(table_or_string, value, 1, true) ~= nil
    end

    local details = passed and nil or (format_value(table_or_string) .. " does not contain " .. format_value(value))
    return print_result(passed, message or "should contain value", details)
end

---@param table_or_string table|string Table or string to check
---@param expected_length number Expected length
---@param message? string Optional message
---@return boolean Assertion result
function assert_length(table_or_string, expected_length, message)
    local actual_length = #table_or_string
    local passed = actual_length == expected_length
    local details = passed and nil or ("expected length " .. expected_length .. ", got " .. actual_length)
    return print_result(passed, message or "should have correct length", details)
end

-- Test organization functions - provide structured test output and scoping

---@param name string Suite name
---@param test_function function Test function to run
function describe(name, test_function)
    print("\n--- " .. name .. " ---")
    local old_stats = {
        passed = test_stats.passed,
        failed = test_stats.failed,
        total = test_stats.total
    }

    test_function()

    local suite_passed = test_stats.passed - old_stats.passed
    local suite_failed = test_stats.failed - old_stats.failed
    local suite_total = test_stats.total - old_stats.total

    print(string.format("--- End %s: %d/%d passed ---\n", name, suite_passed, suite_total))
end

---@param description string Test description
---@param test_function function Test function to run
function it(description, test_function)
    print("  " .. description)
    test_function()
end

---@return boolean True if all tests passed
function print_test_summary()
    print("\n" .. string.rep("=", 50))
    print(string.format("TEST SUMMARY: %d/%d tests passed", test_stats.passed, test_stats.total))
    if test_stats.failed > 0 then
        print(string.format("FAILURES: %d", test_stats.failed))
    end
    print(string.rep("=", 50))
    return test_stats.failed == 0
end

function reset_test_stats()
    test_stats.passed = 0
    test_stats.failed = 0
    test_stats.total = 0
end

---@return table Table with test statistics
function get_test_stats()
    return {
        passed = test_stats.passed,
        failed = test_stats.failed,
        total = test_stats.total
    }
end

-- Creates test doubles with automatic call tracking and verification
-- Enables testing of interactions between components without real implementations
---@param name? string Optional mock name
---@param methods? table Optional table of method implementations
---@return table Mock object
function create_mock(name, methods)
    local mock = {
        __name = name or "Mock",
        __calls = {}
    }

    methods = methods or {}

    for method_name, method_func in pairs(methods) do
        mock[method_name] = function(self, ...)
            table.insert(self.__calls, { method = method_name, args = { ... } })
            if type(method_func) == "function" then
                return method_func(self, ...)
            else
                return method_func
            end
        end
    end

    mock.was_called = function(self, method_name)
        for _, call in ipairs(self.__calls) do
            if call.method == method_name then
                return true
            end
        end
        return false
    end

    mock.call_count = function(self, method_name)
        local count = 0
        for _, call in ipairs(self.__calls) do
            if call.method == method_name then
                count = count + 1
            end
        end
        return count
    end

    return mock
end

-- Module table for explicit require() usage (functions are also available globally)
local TestUtils = {
    assert_eq = assert_eq,
    assert_not_eq = assert_not_eq,
    assert_true = assert_true,
    assert_false = assert_false,
    assert_not_nil = assert_not_nil,
    assert_nil = assert_nil,
    assert_type = assert_type,
    assert_greater_than = assert_greater_than,
    assert_greater_or_equal = assert_greater_or_equal,
    assert_less_than = assert_less_than,
    assert_less_or_equal = assert_less_or_equal,
    assert_contains = assert_contains,
    assert_length = assert_length,
    describe = describe,
    it = it,
    print_test_summary = print_test_summary,
    reset_test_stats = reset_test_stats,
    get_test_stats = get_test_stats,
    create_mock = create_mock
}

if not _G.bevy_log then
    _G.bevy_log = function(args)
        -- Silent mock for testing
    end
end

return TestUtils
