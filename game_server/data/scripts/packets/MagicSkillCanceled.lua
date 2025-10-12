---@class MagicSkillCanceled
local MagicSkillCanceled = {}

-- Sends a MagicSkillCanceled packet for the specified entity
---@param entity Entity The entity whose skill is being canceled
function MagicSkillCanceled.send(entity)
    local object_id = world.get_component(entity, types.ObjectId)
    local magic_skill_canceled_packet = construct(types.MagicSkillCanceled, {
        _1 = object_id
    })

    local gs_packet = construct(types.GameServerPacket,
        { variant = "MagicSkillCanceled", _1 = magic_skill_canceled_packet })

    local broadcastScope = construct(types.BroadcastScope, { variant = "KnownAndSelf" })
    GameServerPacket.broadcast({ entity, gs_packet, broadcastScope })
end

return MagicSkillCanceled
