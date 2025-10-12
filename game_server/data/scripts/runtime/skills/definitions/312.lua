require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 312,
    levels = 20,
    name = "Vicious Stance",
    description = "Increases Critical Damages by 35. Continuously consumes MP proportionately to the user's level.",
    kind = "Toggle",
    tables = {
        abnormalLevels = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 },
        criticalDamage = { 35, 48, 64, 84, 109, 139, 166, 196, 229, 266, 306, 349, 379, 410, 443, 475, 509, 542, 576, 609 },
        magicLevel = { 20, 24, 28, 32, 36, 40, 43, 46, 49, 52, 55, 58, 60, 62, 64, 66, 68, 70, 72, 74 },
        mpInitialConsume = { 4, 5, 5, 6, 7, 7, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 13, 14, 14 },
        stats = {
            CriticalDamage = { "Add", { 35, 48, 64, 84, 109, 139, 166, 196, 229, 266, 306, 349, 379, 410, 443, 475, 509, 542, 576, 609 } },
        },
    },
    other = {
        abnormalKind = "CriticalDmgUpToggle",
        icon = "icon.skill0312",
        targetType = "Self",
        overTimeEffects = {
            Mp = { "Sub", { 0 }, 0.8, 3.33 },
        },
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
