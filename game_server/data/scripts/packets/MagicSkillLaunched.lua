---@class MagicSkillLaunched
local MagicSkillLaunched = {}

-- Sends a MagicSkillLaunched packet for the specified entity
---@param entity Entity The entity casting the skill
---@param skill_ref Skill Reference to the skill being launched
---@param targets table Table of entity targets for the skill
function MagicSkillLaunched.send(entity, skill_ref, targets)
    local object_id = world.get_component(entity, types.ObjectId)
    local magic_skill_launched_packet = construct(types.MagicSkillLaunched,
        {
            object_id = object_id,
            skill = skill_ref,
            targets = targets
        })

    local gs_packet = construct(types.GameServerPacket,
        { variant = "MagicSkillLaunched", _1 = magic_skill_launched_packet })

    local broadcastScope = construct(types.BroadcastScope, { variant = "KnownAndSelf" })
    GameServerPacket.broadcast({ entity, gs_packet, broadcastScope })
end

return MagicSkillLaunched
