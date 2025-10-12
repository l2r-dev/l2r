require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 1085,
    levels = 3,
    name = "Acumen",
    description = "Increases Casting Spd. by 15% for 20 minutes.",
    kind = "Active",
    tables = {
        abnormalLevels = { 1, 2, 3 },
        effectPoints = { 204, 331, 457 },
        mpConsume = { 16, 24, 35 },
        mpInitialConsume = { 4, 6, 9 },
        stats = {
            CastSpd = { "Mul", { 1.15, 1.23, 1.3 } },
        },
    },
    other = {
        abnormalTime = 1200000, -- 20 minutes in milliseconds
        abnormalKind = "CastingTimeDown",
        castRange = 400,
        effectRange = 900,
        hitTime = 4000,
        reuseDelay = 2000,
        icon = "icon.skill1085",
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
