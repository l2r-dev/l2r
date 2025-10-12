require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 77,
    levels = 2,
    name = "Attack Aura",
    description = "Increases P. Atk. by 8% for 20 minutes.",
    kind = "Active",
    tables = {
        abnormalLevels = { 1, 2 },
        effectPoints = { 138, 268 },
        magicLevel = { 10, 28 },
        mpConsume = { 5, 10 },
        mpInitialConsume = { 2, 3 },
        stats = {
            PAtk = { "Mul", { 1.08, 1.12 } }
        }
    },
    other = {
        abnormalTime = 1200000, -- 20 minutes in milliseconds (1200 seconds * 1000)
        abnormalKind = "PaUp",
        hitTime = 4000,
        reuseDelay = 2000,
        icon = "icon.skill0077",
        isMagic = true,
        targetType = "Self",
        priority = 2
    },
}

---@type SkillHandler
local Skill = {
    definition = definition,
    pend = function(entity, skill_ref, shift_pressed, ctrl_pressed)
        Target.pend_on_self(entity, skill_ref)
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
