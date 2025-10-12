require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 1216,
    levels = 1,
    name = "Self Heal",
    description = "Restores one's HP with 42 Power.",
    kind = "Active",
    tables = {
        effectPoints = { 10 },
        magicLevel = { 1 },
        mpConsume = { 7 },
        mpInitialConsume = { 2 },
        power = { 42 },
    },
    other = {
        hitTime = 5000,
        reuseDelay = 3000,
        icon = "icon.skill1216",
        isMagic = true,
        targetType = "Self",
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
        AbnormalEffects.launch_restore_skill(entity, entity, skill_ref, definition, "Hp")
    end,
}

return Skill
