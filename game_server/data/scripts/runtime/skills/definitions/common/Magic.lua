require("data.scripts.Utils")

local Stats = req("data.scripts.game.Stats")
local SystemMessage = req("data.scripts.packets.SystemMessage")
local SkillReuse = req("data.scripts.game.SkillReuse")
local MagicSkillUse = req("data.scripts.packets.MagicSkillUse")
local Casting = req("data.scripts.game.Casting")
local Spatial = req("data.scripts.game.Spatial")
local Attacking = req("data.scripts.game.Attacking")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")
local PathFinding = req("data.scripts.game.PathFinding")
local Movement = req("data.scripts.game.Movement")
local PathfindingTimer = req("data.scripts.game.InActionPathfindingTimer")


local Magic = {}

---@param entity Entity
---@param pending_skill LuaPendingSkillData
---@param skill_definition SkillDefinition
local function handle_pending_self_magic(entity, pending_skill, skill_definition)
    local skill_level = pending_skill.skill_ref.level._1
    local self_transform = world.get_component(entity, types.Transform)

    -- Check MP
    local mp_initial = skill_definition.tables.mpInitialConsume[skill_level]
    local mp_consume = skill_definition.tables.mpConsume and skill_definition.tables.mpConsume[skill_level] or 0
    local total_mp_needed = mp_initial + mp_consume

    local cast_allowed = Stats.check_mp(entity, total_mp_needed)

    if cast_allowed then
        Stats.consume_mp(entity, mp_initial)

        -- Calculate actual cast time based on CastSpd (for magic) or PAtkSpd (for physical)
        local cast_time = Stats.calc_cast_time(entity, skill_definition)

        -- Calculate actual reuse delay based on stats
        local reuse_delay = Stats.calc_reuse_delay(entity, skill_definition)

        MagicSkillUse.send(
            entity,
            entity,         -- Self targeting
            self_transform,
            self_transform, -- Same transform for source and target
            nil,
            pending_skill.skill_ref,
            cast_time,
            reuse_delay
        )

        -- Start reuse timer
        SkillReuse.start_timer(entity, pending_skill.skill_ref, reuse_delay)

        -- Start casting
        Casting.insert_component(entity, entity, cast_time, pending_skill.skill_ref)

        -- Remove pending skill
        world.remove_component(entity, types.LuaPendingSkill)
        return true
    else
        -- Not enough MP
        SystemMessage.send(entity, 24, {})
        world.remove_component(entity, types.LuaPendingSkill)
        return false
    end
end

---@param entity Entity
---@param pending_skill LuaPendingSkillData
---@param skill_definition SkillDefinition
function Magic.handle_pending_self_toggle(entity, pending_skill, skill_definition)
    local skill_level = pending_skill.skill_ref.level._1
    local self_transform = world.get_component(entity, types.Transform)

    -- Check MP
    local cast_allowed = true
    local has_effect = AbnormalEffect.has_effect({ entity, pending_skill.skill_ref.id })
    if not has_effect then
        local mp_initial = skill_definition.tables.mpInitialConsume[skill_level]
        cast_allowed = Stats.check_mp(entity, mp_initial)
        if cast_allowed then
            Stats.consume_mp(entity, mp_initial)
        end
    end

    if cast_allowed then
        -- Calculate actual cast time based on CastSpd (for magic) or PAtkSpd (for physical)
        local cast_time = Stats.calc_cast_time(entity, skill_definition)

        -- Calculate actual reuse delay based on stats
        local reuse_delay = Stats.calc_reuse_delay(entity, skill_definition)

        MagicSkillUse.send(
            entity,
            entity,         -- Self targeting
            self_transform,
            self_transform, -- Same transform for source and target
            nil,
            pending_skill.skill_ref,
            cast_time,
            reuse_delay
        )

        -- Start reuse timer
        SkillReuse.start_timer(entity, pending_skill.skill_ref, reuse_delay)

        -- Start casting
        Casting.insert_component(entity, entity, cast_time, pending_skill.skill_ref)

        -- Remove pending skill
        world.remove_component(entity, types.LuaPendingSkill)
        return true
    else
        -- Not enough MP
        SystemMessage.send(entity, 24, {})
        world.remove_component(entity, types.LuaPendingSkill)
        return false
    end
end

---@param entity Entity
---@param pending_skill LuaPendingSkillData
---@param skill_definition SkillDefinition
local function handle_pending_magic(entity, pending_skill, skill_definition)
    if Casting.in_progress(entity) then
        return
    end

    -- Skip if entity has pathfinding cooldown timer (prevents spam)
    if PathfindingTimer.has_component(entity) then
        return false
    end

    local skill_level = pending_skill.skill_ref.level._1
    local selected_target_component = world.get_component(entity, types.SelectedTarget)

    if not selected_target_component then
        world.remove_component(entity, types.LuaPendingSkill)
        return false
    end

    local target_entity = selected_target_component._1:clone()

    if target_entity == nil then
        world.remove_component(entity, types.LuaPendingSkill)
        return false
    end

    local self_transform = world.get_component(entity, types.Transform)
    local target_transform = world.get_component(target_entity, types.Transform)

    local distance = Spatial.flat_distance(self_transform.translation, target_transform.translation)

    if distance > skill_definition.other.castRange then
        -- Check if already moving to the correct target
        if Movement.is_moving_to_entity(entity, target_entity) then
            -- Already moving to target, continue waiting
            return false
        end

        -- For magic skills, first check if we can see the target (magic can cast over low obstacles)
        local can_see = RegionGeoData.can_see_target({
            self_transform.translation,
            target_transform.translation
        })

        if can_see then
            -- Can see target, use simple movement (magic can cast over low obstacles)
            Movement.to_entity(entity, target_entity, skill_definition.other.castRange)
        else
            -- Can't see target, use pathfinding system with cooldown timer
            PathfindingTimer.insert_component(entity)
            PathFinding.request_visibility_check(
                entity,
                self_transform.translation,
                target_transform.translation
            )
        end
        return false
    else
        -- Within range, check line of sight first
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

        -- Within range and can see target, check MP
        local mp_initial = skill_definition.tables.mpInitialConsume[skill_level]
        local mp_consume = skill_definition.tables.mpConsume and skill_definition.tables.mpConsume[skill_level] or 0
        local total_mp_needed = mp_initial + mp_consume

        local cast_allowed = Stats.check_mp(entity, total_mp_needed)

        if cast_allowed then
            Stats.consume_mp(entity, mp_initial)

            -- Calculate actual cast time based on CastSpd (for magic) or PAtkSpd (for physical)
            local cast_time = Stats.calc_cast_time(entity, skill_definition)

            -- Calculate actual reuse delay based on stats
            local reuse_delay = Stats.calc_reuse_delay(entity, skill_definition)


            -- Send MagicSkillUse packet
            MagicSkillUse.send(
                entity,
                target_entity,
                self_transform,
                target_transform,
                nil,
                pending_skill.skill_ref,
                cast_time,
                reuse_delay
            )

            -- Start reuse timer
            SkillReuse.start_timer(entity, pending_skill.skill_ref, reuse_delay)

            -- Start casting
            Casting.insert_component(entity, target_entity, cast_time, pending_skill.skill_ref)

            -- Stop moving
            Movement.remove(entity)

            -- Remove pending skill
            world.remove_component(entity, types.LuaPendingSkill)
            return true
        else
            -- Not enough MP
            SystemMessage.send(entity, 24, {})
            world.remove_component(entity, types.LuaPendingSkill)
            Movement.remove(entity)
            return false
        end
    end
end

---Executes a magical bolt skill that deals damage to a target
---@param entity Entity The caster entity
---@param target_entity Entity The target entity
---@param skill_ref Skill The skill reference
---@param skill_definition SkillDefinition The skill definition containing power values
function Magic.launch_magical_bolt_skill(entity, target_entity, skill_ref, skill_definition)
    local skill_level = skill_ref.level._1
    local skill_power = skill_definition.tables.power[skill_level]

    -- Consume final MP
    local mp_consume = skill_definition.tables.mpConsume[skill_level]
    Stats.consume_mp(entity, mp_consume)

    local damage = Magic.magical_damage(entity, target_entity, skill_level, skill_definition)

    Stats.apply_damage(target_entity, damage)

    Attacking.insert_component(target_entity, entity)

    -- Send system message about damage dealt
    SystemMessage.send_with_number(entity, 35, damage)
end

---@param entity Entity The caster entity
---@param target_entity Entity The target entity
---@param skill_level number The level of the skill
---@param skill_definition SkillDefinition The skill definition
---@return number damage The calculated damage amount
function Magic.magical_damage(entity, target_entity, skill_level, skill_definition)
    -- TODO: Implement full magical damage calculation considering M.Atk, resistances, etc.
    return skill_definition.tables.power[skill_level]
end

---@param entity Entity The entity casting the skill
---@param target_entity Entity The entity receiving the buff
---@param skill_ref Skill The skill reference containing level and other details
---@param skill_definition SkillDefinition The skill definition containing tables and other properties
function Magic.launch_buff_skill(entity, target_entity, skill_ref, skill_definition)
    -- Consume final MP
    local mp_consume = skill_definition.tables.mpConsume[skill_ref.level._1]
    Stats.consume_mp(entity, mp_consume)

    -- Create effects over time from overTimeEffects definition (if any)
    local effects_over_time = AbnormalEffects.create_effects_over_time(target_entity, skill_ref,
        skill_definition.other.overTimeEffects)

    -- Create and apply the abnormal effect
    local effect, timer_data = AbnormalEffects.effect(
        skill_ref,
        skill_definition.other.abnormalTime,
        skill_definition.other.abnormalKind,
        effects_over_time
    )
    AbnormalEffects.apply_effect(target_entity, effect, timer_data, skill_definition)
end

---Event handler for pending magic skills
---@param entity Entity The entity with the pending skill
---@param pending_skill LuaPendingSkillData The PendingSkill component data
---@param skill_definition SkillDefinition The skill definition
function Magic.on_pending_skill(entity, pending_skill, skill_definition)
    if skill_definition.other.targetType == "Self" then
        handle_pending_self_magic(entity, pending_skill, skill_definition)
    else
        handle_pending_magic(entity, pending_skill, skill_definition)
    end
end

return Magic
