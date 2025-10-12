local Utils = {}

-- For advanced logging functionality, see Logger.lua module

---@param tbl table The table to search in
---@param value any The value to search for
---@return boolean true if the value is found, false otherwise
function Utils.contains(tbl, value)
    for _, v in ipairs(tbl) do
        if v == value then
            return true
        end
    end
    return false
end

---@param t table The table to count
---@return number The number of elements in the table
function Utils.table_count(t)
    local count = 0
    for _ in pairs(t) do
        count = count + 1
    end
    return count
end

---Function to wrap original require to cleanup module cache, suitaible for development
---@param modname string
---@return unknown
---@return unknown loaderdata
function Utils.require(modname)
    package.loaded[modname] = nil
    return require(modname)
end

--- Pretty prints a table with proper formatting and indentation
---@param t table The table to print
---@param indent? number Optional indentation level (default: 0)
---@param max_depth? number Optional maximum depth to prevent infinite recursion (default: 5)
---@return string String representation of the table
function Utils.table_to_string(t, indent, max_depth)
    indent = indent or 0
    max_depth = max_depth or 5

    if indent > max_depth then
        return "... (max depth reached)"
    end

    if type(t) ~= "table" then
        if type(t) == "string" then
            return '"' .. tostring(t) .. '"'
        else
            return tostring(t)
        end
    end

    local result = "{\n"
    local indent_str = string.rep("  ", indent + 1)
    local closing_indent = string.rep("  ", indent)

    for k, v in pairs(t) do
        local key_str = type(k) == "string" and k or "[" .. tostring(k) .. "]"
        local value_str = Utils.table_to_string(v, indent + 1, max_depth)
        result = result .. indent_str .. key_str .. " = " .. value_str .. ",\n"
    end

    result = result .. closing_indent .. "}"
    return result
end

---@param t table The table to print
---@param name? string Optional name/label for the table
function Utils.print_table(t, name)
    local label = name and (name .. " = ") or ""
    bevy_log({ "info", "📋 " .. label .. Utils.table_to_string(t) })
end

---@param component_name string The name of the component to register
---@return ScriptTypeRegistration type_registration The registered or existing component type
function Utils.register_dyn_component(component_name)
    local existing_component = world.get_type_by_name(component_name)
    if not existing_component then
        local new_component = world.register_new_component(component_name)
        bevy_log({ "info", component_name .. " component registered" })
        return new_component
    else
        return existing_component
    end
end

_G.reload_require = Utils.require
_G.req = Utils.require
_G.contains = Utils.contains
_G.table_count = Utils.table_count
_G.table_to_string = Utils.table_to_string
_G.print_table = Utils.print_table
_G.register_dyn_component = Utils.register_dyn_component



return Utils
