require("data.scripts.types")

local SystemMessage = {}

-- Creates a SmParam of the specified variant type
---@param param_type string The type of parameter (Text, Number, Npc, etc.)
---@param value any The value for the parameter
---@return table The constructed SmParam
function SystemMessage.create_param(param_type, value)
    return construct(types.SmParam, { variant = param_type, _1 = value })
end

-- Sends a SystemMessage packet to a player or broadcast
---@param entity Entity Entity ReflectReference
---@param message_id number The system message ID to send
---@param params? table A table of parameters for the message (SystemMessage.create_param())
---@param broadcast? boolean Whether to broadcast the message
function SystemMessage.send(entity, message_id, params, broadcast)
    local system_message = construct(types.SystemMessage, {
        id = message_id,
        message_params = params
    })

    local gs_packet = construct(types.GameServerPacket, {
        variant = "SystemMessage",
        _1 = system_message
    })

    if not broadcast then
        GameServerPacket.send({ entity, gs_packet })
    else
        -- Broadcast to all known players
        local broadcastScope = construct(types.BroadcastScope, { variant = "KnownAndSelf" })
        GameServerPacket.broadcast({ entity, gs_packet, broadcastScope })
    end
end

--[[
    Convenience function to send a simple system message by ID only

    @param entity The entity to send the message to
    @param message_id number The system message ID
]]
---@param entity Entity The entity to send the message to
---@param message_id number The system message ID
function SystemMessage.send_simple(entity, message_id)
    SystemMessage.send(entity, message_id, {}, false)
end

--[[
    Convenience function to send a simple text system message

    @param target_entity The entity to send the message to (optional, if nil broadcasts to all)
    @param message_id number The system message ID
    @param text string The text to include in the message
]]
---@param entity Entity? The entity to send the message to (optional, if nil broadcasts to all)
---@param message_id number The system message ID
---@param text string The text to include in the message
function SystemMessage.send_with_text(entity, message_id, text)
    local params = { SystemMessage.create_param("Text", text) }
    SystemMessage.send(entity, message_id, params)
end

---@param entity Entity? The entity to send the message to (optional)
---@param message_id number The system message ID
---@param player_name string The player's name to include
function SystemMessage.send_with_player_name(entity, message_id, player_name)
    local params = { SystemMessage.create_param("Player", player_name) }
    SystemMessage.send(entity, message_id, params)
end

--[[
    Convenience function to send a simple numeric system message

    @param target_entity The entity to send the message to (optional, if nil broadcasts to all)
    @param message_id number The system message ID
    @param number number The number to include in the message
]]
---@param entity Entity? The entity to send the message to (optional, if nil broadcasts to all)
---@param message_id number The system message ID
---@param number number The number to include in the message
function SystemMessage.send_with_number(entity, message_id, number)
    local params = { SystemMessage.create_param("Number", number) }
    SystemMessage.send(entity, message_id, params)
end

return SystemMessage
