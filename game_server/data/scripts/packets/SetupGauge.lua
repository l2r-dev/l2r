---@class SetupGauge
local SetupGauge = {}

-- Sends a SetupGauge packet for the specified entity
---@param entity Entity The entity to display the gauge for
---@param color string The color of the gauge ("Blue", "Red", "Cyan", "Green")
---@param current_time number The current time value for the gauge in milliseconds
---@param total_time number The total time value for the gauge in milliseconds
function SetupGauge.send(entity, color, current_time, total_time)
    local color_value = construct(types.SetupGaugeColor, { variant = color })

    local setup_gauge_packet = construct(types.SetupGauge,
        {
            object_id = world.get_component(entity, types.ObjectId),
            color = color_value,
            current_time = Duration.from_millis(current_time),
            total_time = Duration.from_millis(total_time)
        })

    local gs_packet = construct(types.GameServerPacket, { variant = "SetupGauge", _1 = setup_gauge_packet })

    GameServerPacket.send({ entity, gs_packet })
end

return SetupGauge
