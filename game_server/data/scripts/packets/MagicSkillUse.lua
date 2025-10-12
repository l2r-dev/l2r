local SetupGauge = require("data.scripts.packets.SetupGauge")
local MagicSkillLaunched = require("data.scripts.packets.MagicSkillLaunched")

local MagicSkillUse = {}

--- Sends a MagicSkillUse packet for the specified entity and target
---@param entity Entity The entity using the skill
---@param target_entity Entity The target entity of the skill
---@param self_transform Transform The transform component of the entity
---@param target_transform Transform The transform component of the target
---@param aoe_targets Entity[]? A list of targets entities for area of effect skills
---@param skill_ref Skill
---@param hit_time MilliSeconds
---@param reuse_delay MilliSeconds
function MagicSkillUse.send(
    entity,
    target_entity,
    self_transform,
    target_transform,
    aoe_targets,
    skill_ref,
    hit_time,
    reuse_delay
)
    local user_oid = world.get_component(entity, types.ObjectId)
    local target_oid = world.get_component(target_entity, types.ObjectId)
    local hit_time = hit_time or 0
    local reuse_delay = reuse_delay or 0

    local use_skill_packet = construct(types.MagicSkillUse,
        {
            user = user_oid,
            origin_location = self_transform.translation,
            target = target_oid,
            target_location = target_transform.translation,
            skill = skill_ref,
            hit_time = Duration.from_millis(hit_time),
            reuse_delay = Duration.from_millis(reuse_delay),
            ground_location = nil,
        })

    local gs_packet = construct(types.GameServerPacket, { variant = "MagicSkillUse", _1 = use_skill_packet })
    local broadcastScope = construct(types.BroadcastScope, { variant = "KnownAndSelf" })
    GameServerPacket.broadcast({ entity, gs_packet, broadcastScope })

    if aoe_targets == nil then
        aoe_targets = { target_oid }
    else
        -- convert target entities to object id
        for i, target in ipairs(aoe_targets) do
            aoe_targets[i] = world.get_component(target, types.ObjectId)
        end
    end

    MagicSkillLaunched.send(entity, skill_ref, aoe_targets)

    SetupGauge.send(entity, "Blue", hit_time, hit_time)
end

return MagicSkillUse
