local Target = {}

local SystemMessage = require("data.scripts.packets.SystemMessage")

---Checks if an entity is considered an enemy (attackable)
---@param entity Entity The entity to check
---@return boolean is_enemy true if the entity is attackable, false otherwise
function Target.is_enemy(entity)
    local attackable = world.get_component(entity, types.Attackable)
    return attackable ~= nil
end

---Checks if an entity is considered an ally (not attackable)
---@param entity Entity The entity to check
---@return boolean is_ally true if the entity is not attackable, false otherwise
function Target.is_ally(entity)
    return not Target.is_enemy(entity)
end

function Target.validate_target_exists_and_not_self(caster_entity, target_entity)
    if target_entity == nil then
        SystemMessage.send(caster_entity, 109, {}) -- Invalid target
        return false
    elseif target_entity == caster_entity then
        SystemMessage.send(caster_entity, 51, {}) -- Cannot target self
        return false
    else
        local dead = world.get_component(target_entity, types.Dead)
        if dead then
            SystemMessage.send(caster_entity, 109, {}) -- Invalid target
            return false
        end
    end
    return true
end

function Target.validate_target_exists_allow_self(caster_entity, target_entity)
    if target_entity == nil then
        SystemMessage.send(caster_entity, 109, {}) -- Invalid target
        return false
    else
        local dead = world.get_component(target_entity, types.Dead)
        if dead then
            SystemMessage.send(caster_entity, 109, {}) -- Invalid target
            return false
        end
    end
    return true
end

---Validate target for enemy-only skills (magic bolt type)
---@param caster_entity Entity The caster entity
---@param target_entity Entity The target entity
---@param ctrl_pressed boolean Whether ctrl key is pressed
---@return boolean is_valid true if target is valid
function Target.validate_enemy_target(caster_entity, target_entity, ctrl_pressed)
    if not Target.validate_target_exists_and_not_self(caster_entity, target_entity) then
        return false
    end

    -- If ctrl is pressed, allow targeting anyone (including allies)
    if ctrl_pressed then
        return true
    end

    -- Without ctrl, only allow targeting enemies
    if Target.is_ally(target_entity) then
        SystemMessage.send(caster_entity, 109, {}) -- Invalid target (ally without ctrl)
        return false
    end

    return true
end

---Validate target for ally-only skills (buff/restore type)
---@param caster_entity Entity The caster entity
---@param target_entity Entity The target entity
---@param ctrl_pressed boolean Whether ctrl key is pressed
---@return boolean is_valid true if target is valid
function Target.validate_ally_target(caster_entity, target_entity, ctrl_pressed)
    if not Target.validate_target_exists_allow_self(caster_entity, target_entity) then
        return false
    end

    -- If ctrl is pressed, allow targeting enemies too
    if ctrl_pressed then
        return true
    end

    -- Without ctrl, only allow targeting allies (including self)
    if target_entity ~= caster_entity and Target.is_enemy(target_entity) then
        SystemMessage.send(caster_entity, 109, {}) -- Invalid target (enemy without ctrl)
        return false
    end

    return true
end

---Initiates a skill targeting an enemy, validating the skill first
---@param entity Entity The entity requesting to use the skill
---@param skill_ref Skill The skill reference
---@param skill_definition SkillDefinition The skill definition
---@param ctrl_pressed boolean Whether ctrl key is pressed
---@param _shift_pressed boolean Whether shift key is pressed (unused)
function Target.pend_skill_on_enemy(entity, skill_ref, skill_definition, ctrl_pressed, _shift_pressed)
    local selected_target_component = world.get_component(entity, types.SelectedTarget)
    if not selected_target_component then
        SystemMessage.send(entity, 109, {}) -- Invalid target
        return
    end

    local selected_target = selected_target_component._1:clone()
    if selected_target then
        if Target.validate_enemy_target(entity, selected_target, ctrl_pressed) then
            Target.insert_component(entity, selected_target, skill_ref)
        end
    else
        SystemMessage.send(entity, 109, {}) -- Invalid target
    end
end

---Initiates a skill targeting self or an ally, validating the skill first
---@param entity Entity The entity requesting to use the skill
---@param skill_ref Skill The skill reference
---@param ctrl_pressed boolean Whether ctrl key is pressed (allows targeting enemies)
---@param _shift_pressed boolean Whether shift key is pressed (unused)
---@param skill_definition SkillDefinition The skill definition to check targetType
function Target.pend_skill_on_self_or_ally(entity, skill_ref, ctrl_pressed, _shift_pressed, skill_definition)
    local selected_target_component = world.get_component(entity, types.SelectedTarget)
    local is_self_only = skill_definition and skill_definition.other.targetType == "Self"

    -- If skill is self-only, always target self
    if is_self_only then
        Target.insert_component(entity, entity, skill_ref)
        return
    end

    -- For target-able skills, check if we have a target
    if not selected_target_component then
        SystemMessage.send(entity, 109, {}) -- Invalid target (no target selected)
        return
    end

    local selected_target = selected_target_component._1:clone()

    -- If no target selected
    if not selected_target then
        SystemMessage.send(entity, 109, {}) -- Invalid target (no target selected)
        return
    end

    -- Validate ally target (ctrl_pressed allows targeting enemies too)
    if Target.validate_ally_target(entity, selected_target, ctrl_pressed) then
        Target.insert_component(entity, selected_target, skill_ref)
    else
        -- Invalid target
        SystemMessage.send(entity, 109, {}) -- Invalid target
    end
end                                         --- New skills

function Target.pend_on_self(entity, skill_ref)
    Target.insert_component(entity, entity, skill_ref)
end

---Adds a pending skill component to mark that an entity wants to use a skill
---@param entity Entity The entity using the skill
---@param target_entity Entity The target entity
---@param skill_ref Skill The skill reference
function Target.insert_component(entity, target_entity, skill_ref)
    ---@type LuaPendingSkillData
    local pending_data = {
        caster_entity = entity,
        target_entity = target_entity,
        skill_ref = skill_ref,
    }

    world.insert_component(entity, types.LuaPendingSkill, construct(types.DynamicComponent, {
        data = pending_data
    }))
end

return Target
