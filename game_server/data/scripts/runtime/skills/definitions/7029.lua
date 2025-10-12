require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

local speedEffect = { "Add", { 75, 150, 300 } }
local attackSpeedEffect = { "Mul", { 1.5, 2.0, 3.0 } }

---@type SkillDefinition
local definition = {
    id = 7029,
    levels = 3,
    name = "Super Haste",
    description = "Admin Haste",
    kind = "Toggle",
    tables = {
        abnormalLevels = { 1, 2, 3 },
        magicLevel = { 80, 80, 80 },
        mpInitialConsume = { 1, 1, 1 },
        stats = {
            Walk = speedEffect,
            Run = speedEffect,
            Swim = speedEffect,
            FastSwim = speedEffect,
            PAtkSpd = attackSpeedEffect,
            CastSpd = attackSpeedEffect,
        },
    },
    other = {
        abnormalKind = "SuperHasteToggle",
        icon = "icon.skill7029",
        targetType = "Self",
        priority = 3,
    },
}

---@type SkillHandler
local Skill = {
    definition = definition,
    pend = function(entity, skill_ref, shift_pressed, ctrl_pressed)
        Target.pend_on_self(entity, skill_ref)
    end,
    on_pending = function(entity, pending_skill)
        Magic.handle_pending_self_toggle(entity, pending_skill, definition)
    end,
    launch = function(entity, target_entity, skill_ref)
        AbnormalEffects.launch_toggle_skill(entity, skill_ref, definition)
    end,
    apply_abnormal = function(target_entity, skill_ref)
        AbnormalEffects.apply_stat_modifiers(target_entity, definition, skill_ref.level._1)
    end,
}
return Skill
