local DaggerBlow = {}

local Stats = req("data.scripts.game.Stats")
local SystemMessage = req("data.scripts.packets.SystemMessage")
local SkillReuse = req("data.scripts.game.SkillReuse")

local PaperDoll = req("data.scripts.game.PaperDoll")
local MagicSkillUse = req("data.scripts.packets.MagicSkillUse")
local Casting = req("data.scripts.game.Casting")
local Attacking = req("data.scripts.game.Attacking")
local PlaySound = req("data.scripts.packets.PlaySound")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local PathFinding = req("data.scripts.game.PathFinding")
local Movement = req("data.scripts.game.Movement")
local PathfindingTimer = req("data.scripts.game.InActionPathfindingTimer")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("skills.DaggerBlow")

---@param entity Entity The attacking entity
---@param target_entity Entity The target entity
---@param skill_level number The level of the skill
---@param dskill_definition SkillDefinition The skill definition
---@return number damage The calculated damage amount
function DaggerBlow.blow_damage(entity, target_entity, skill_level, dskill_definition)
    local damage = dskill_definition.tables.power[skill_level]
    return damage
end

---@param entity Entity The entity using the skill
---@param skill_ref Skill The skill reference with ID and level
---@param skill_definition SkillDefinition The skill definition containing requirements
---@param shift_pressed boolean Whether shift key is pressed
---@param ctrl_pressed boolean Whether ctrl key is pressed
function DaggerBlow.pend(entity, skill_ref, skill_definition, shift_pressed, ctrl_pressed)
    if not PaperDoll.has_required_weapon(
            entity,
            skill_definition.conditions.weaponRequired,
            skill_definition.conditions.weaponExtraKind) then
        SystemMessage.send_with_text(entity, 113, skill_definition.name)
        return
    end

    local selected_target = world.get_component(entity, types.SelectedTarget)._1:clone()

    if Target.validate_target_exists_and_not_self(entity, selected_target) then
        Target.insert_component(entity, selected_target, skill_ref)
    else
        return
    end
end

---@param entity Entity The attacking entity
---@param target_entity Entity The target entity
---@param skill_ref Skill The skill reference
---@param dskill_definition SkillDefinition The skill definition
function DaggerBlow.launch_skill(entity, target_entity, skill_ref, dskill_definition)
    local shld_result = ShieldResult.calculate({ entity, target_entity })
    print("Shield result: " .. tostring(shld_result))

    local damage = DaggerBlow.blow_damage(entity, target_entity, skill_ref.level._1, dskill_definition)

    Stats.apply_damage(target_entity, damage)

    Attacking.insert_component(target_entity, entity)

    -- TODO: send only if hit
    PlaySound.send(entity, "SkillSound.critical_hit_02")

    local name = world.get_component(entity, types.Name).name
    SystemMessage.send_with_player_name(entity, 2266, name)
    SystemMessage.send_with_number(entity, 35, damage)
end

-- Handles pending dagger blow skill execution including movement, MP consumption, and casting
---@param entity Entity The entity with the pending skill
---@param pending_skill LuaPendingSkillData The PendingSkill component data
---@param skill_definition SkillDefinition The skill definition
---@return boolean|nil true if the skill was processed (success or failure), false if still pending
function DaggerBlow.handle_pending_blow(entity, pending_skill, skill_definition)
    if Casting.in_progress(entity) then
        return
    end

    -- Skip if entity has pathfinding cooldown timer (prevents spam)
    if PathfindingTimer.has_component(entity) then
        return false
    end

    local target_entity = pending_skill.target_entity

    if target_entity == nil then
        world.remove_component(entity, types.LuaPendingSkill)
        return
    else
        local self_transform = world.get_component(entity, types.Transform)
        local target_transform = world.get_component(target_entity, types.Transform)

        local distance = self_transform.translation:distance(target_transform.translation)

        if distance > skill_definition.other.castRange then
            -- Check if already moving to the correct target
            if Movement.is_moving_to_entity(entity, target_entity) then
                -- Already moving to target, continue waiting
                return false
            end

            -- Check if movement to target is possible using geodata (line of sight)
            local can_move = RegionGeoData.can_move_to({
                self_transform.translation,
                target_transform.translation
            })

            if can_move then
                -- Direct line of sight, use simple movement
                Movement.to_entity(entity, target_entity, skill_definition.other.castRange)
            else
                -- No direct line of sight, use pathfinding system with cooldown timer
                PathfindingTimer.insert_component(entity)
                PathFinding.request_visibility_check(
                    entity,
                    self_transform.translation,
                    target_transform.translation
                )
            end
            return false
        else
            -- Within range, check line of sight before casting
            local can_see = RegionGeoData.can_see_target({
                self_transform.translation,
                target_transform.translation
            })

            if not can_see then
                -- Cannot see target even though within range, try pathfinding to find better position
                PathfindingTimer.insert_component(entity)
                PathFinding.request_visibility_check(
                    entity,
                    self_transform.translation,
                    target_transform.translation
                )
                return false
            end

            -- Within range and can see target, proceed with skill
            local skill_ref = pending_skill.skill_ref
            local mp_consumed = skill_definition.tables.mpConsume[skill_ref.level._1]
            local cast_allowed = Stats.consume_mp(entity, mp_consumed)

            if cast_allowed then
                -- Calculate actual hit time based on PAtkSpd
                local hit_time = Stats.calc_cast_time(entity, skill_definition)

                -- Calculate actual reuse delay based on stats
                local reuse_delay = Stats.calc_reuse_delay(entity, skill_definition)

                MagicSkillUse.send(
                    entity,
                    target_entity,
                    self_transform,
                    target_transform,
                    nil,
                    skill_ref,
                    hit_time,
                    reuse_delay
                )

                -- Start reuse timer
                SkillReuse.start_timer(entity, skill_ref, reuse_delay)

                -- Start casting
                Casting.insert_component(entity, target_entity, hit_time,
                    skill_ref)
                -- Stop moving
                MovementHelper.remove(entity)
                -- Attack after casting
                Attacking.insert_component(entity, target_entity)
                world.remove_component(entity, types.LuaPendingSkill)
            else
                SystemMessage.send(entity, 24, {}) -- Not enough MP
                MovementHelper.remove(entity)
                Attacking.insert_component(entity, target_entity)
                world.remove_component(entity, types.LuaPendingSkill)
            end
            return true
        end
    end
end

return DaggerBlow
