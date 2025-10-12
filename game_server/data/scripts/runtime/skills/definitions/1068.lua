require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 1068,
    levels = 3,
    name = "Might",
    description = "Increases P. Atk. by 8% for 20 minutes.",
    kind = "Active",
    tables = {
        abnormalLevels = { 1, 2, 3 },
        effectPoints = { 121, 204, 379 },
        magicLevel = { 7, 20, 40 },
        mpConsume = { 8, 16, 28 },
        mpInitialConsume = { 2, 4, 7 },
        stats = {
            PAtk = { "Mul", { 1.08, 1.12, 1.15 } },
        },
    },
    other = {
        abnormalTime = 1200000, -- 20 minutes in milliseconds
        abnormalKind = "PaUp",
        castRange = 400,
        effectRange = 900,
        hitTime = 4000,
        reuseDelay = 2000,
        icon = "icon.skill1068",
        isMagic = true,
        targetType = "One",
        priority = 2,
    },
}

---@type SkillHandler
local Skill = {
    definition = definition,
    pend = function(entity, skill_ref, shift_pressed, ctrl_pressed)
        Target.pend_skill_on_self_or_ally(entity, skill_ref, ctrl_pressed, shift_pressed, definition)
    end,
    on_pending = function(entity, pending_skill)
        Magic.on_pending_skill(entity, pending_skill, definition)
    end,
    launch = function(entity, target_entity, skill_ref)
        Magic.launch_buff_skill(entity, target_entity, skill_ref, definition)
    end,
    apply_abnormal = function(target_entity, skill_ref)
        AbnormalEffects.apply_stat_modifiers(target_entity, definition, skill_ref.level._1)
    end,
}
return Skill
