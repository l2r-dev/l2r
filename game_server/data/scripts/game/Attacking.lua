---@class Attacking
local Attacking = {}

-- Inserts an Attacking component to mark an entity as attacking another
---@param attacker_entity Entity The entity that will be attacking
---@param target_entity Entity The entity being attacked
function Attacking.insert_component(attacker_entity, target_entity)
    local attacking = construct(types.Attacking, { _1 = target_entity })
    world.insert_component(attacker_entity, types.Attacking, attacking)
end

return Attacking
