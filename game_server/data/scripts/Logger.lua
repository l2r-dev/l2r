local Logger = {}

-- Log levels (in order of severity)
Logger.LEVELS = {
    TRACE = 1,
    DEBUG = 2,
    INFO = 3,
    WARN = 4,
    ERROR = 5
}

-- Level names for display
Logger.LEVEL_NAMES = {
    [Logger.LEVELS.TRACE] = "trace",
    [Logger.LEVELS.DEBUG] = "debug",
    [Logger.LEVELS.INFO] = "info",
    [Logger.LEVELS.WARN] = "warn",
    [Logger.LEVELS.ERROR] = "error"
}

-- Default log level (DEBUG)
Logger.default_level = Logger.LEVELS.DEBUG

-- Internal function to check if a log level should be output
---@param level number The log level to check
---@return boolean true if the level should be logged, false otherwise
local function should_log(level)
    return level >= Logger.default_level
end

-- Global variable to track current script name (set by script loader)
Logger._script_name = "unknown"

-- Internal function to get caller information
-- Since bevy_mod_scripting executes scripts as strings, debug.getinfo won't show original filenames
-- We'll use the manually set script name
---@return string The filename of the caller
local function get_caller_info()
    -- Return the manually set script name
    return Logger._script_name
end

-- Function to set the current script name (should be called at the top of each script)
---@param script_name string The name of the current script
function Logger.set_script_name(script_name)
    Logger._script_name = script_name
end

-- Internal function to format log message with caller info
---@param level number The log level
---@param message any The message to format
---@param prefix? string Optional prefix for the message
---@return string Formatted log message
local function format_message(level, message, prefix)
    local caller = get_caller_info()
    local caller_str = "[" .. caller .. "] "
    local prefix_str = prefix and ("[" .. prefix .. "] ") or ""
    return caller_str .. prefix_str .. tostring(message)
end

-- Logs a message at TRACE level
---@param message any The message to log
---@param prefix? string Optional prefix for the message
function Logger.trace(message, prefix)
    if should_log(Logger.LEVELS.TRACE) then
        bevy_log({ Logger.LEVEL_NAMES[Logger.LEVELS.TRACE], format_message(Logger.LEVELS.TRACE, message, prefix) })
    end
end

-- Logs a message at DEBUG level
---@param message any The message to log
---@param prefix? string Optional prefix for the message
function Logger.debug(message, prefix)
    if should_log(Logger.LEVELS.DEBUG) then
        bevy_log({ Logger.LEVEL_NAMES[Logger.LEVELS.DEBUG], format_message(Logger.LEVELS.DEBUG, message, prefix) })
    end
end

-- Logs a message at INFO level
---@param message any The message to log
---@param prefix? string Optional prefix for the message
function Logger.info(message, prefix)
    if should_log(Logger.LEVELS.INFO) then
        bevy_log({ Logger.LEVEL_NAMES[Logger.LEVELS.INFO], format_message(Logger.LEVELS.INFO, message, prefix) })
    end
end

-- Logs a message at WARN level
---@param message any The message to log
---@param prefix? string Optional prefix for the message
function Logger.warn(message, prefix)
    if should_log(Logger.LEVELS.WARN) then
        bevy_log({ Logger.LEVEL_NAMES[Logger.LEVELS.WARN], format_message(Logger.LEVELS.WARN, message, prefix) })
    end
end

-- Logs a message at ERROR level
---@param message any The message to log
---@param prefix? string Optional prefix for the message
function Logger.error(message, prefix)
    if should_log(Logger.LEVELS.ERROR) then
        bevy_log({ Logger.LEVEL_NAMES[Logger.LEVELS.ERROR], format_message(Logger.LEVELS.ERROR, message, prefix) })
    end
end

-- Sets the minimum log level
---@param level number The minimum log level (use Logger.LEVELS constants)
function Logger.set_level(level)
    if level >= Logger.LEVELS.TRACE and level <= Logger.LEVELS.ERROR then
        Logger.default_level = level
        Logger.info("Log level set to " .. Logger.LEVEL_NAMES[level])
    else
        Logger.error("Invalid log level: " .. tostring(level))
    end
end

-- Gets the current log level
---@return number The current minimum log level
function Logger.get_level()
    return Logger.default_level
end

-- Gets the current log level name
---@return string The current minimum log level name
function Logger.get_level_name()
    return Logger.LEVEL_NAMES[Logger.default_level]
end

-- Logs a table in a readable format at the specified level
---@param t table The table to log
---@param level? number The log level (default: DEBUG)
---@param name? string Optional name/label for the table
function Logger.log_table(t, level, name)
    level = level or Logger.LEVELS.DEBUG
    if should_log(level) then
        local label = name and (name .. " = ") or ""
        local table_str = _G.table_to_string and _G.table_to_string(t) or tostring(t)
        local message = label .. table_str

        bevy_log({ Logger.LEVEL_NAMES[level], format_message(level, message) })
    end
end

-- Logs execution time of a function
---@param func function The function to execute and time
---@param message? string Optional message to include with the timing
---@param level? number Log level for the timing message (default: DEBUG)
---@return any The result of the function call
function Logger.time_execution(func, message, level)
    level = level or Logger.LEVELS.DEBUG
    if not should_log(level) then
        return func()
    end

    local start_time = os.clock()
    local result = func()
    local end_time = os.clock()
    local execution_time = (end_time - start_time) * 1000 -- Convert to milliseconds

    local timing_message = string.format("%s took %.2f ms",
        message or "Function execution", execution_time)

    bevy_log({ Logger.LEVEL_NAMES[level], format_message(level, timing_message) })

    return result
end

-- Default log function that logs at the default level (DEBUG)
---@param message any The message to log
---@param prefix? string Optional prefix for the message
function Logger.log(message, prefix)
    Logger.debug(message, prefix)
end

-- The main global log function - logs to default level (DEBUG)
_G.log = Logger.log

return Logger
