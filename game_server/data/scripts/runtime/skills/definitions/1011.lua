require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 1011,
    levels = 18,
    name = "Heal",
    description = "Restores the target's HP with Power.",
    kind = "Active",
    tables = {
        effectPoints = { 50, 58, 67, 83, 95, 107, 121, 135, 151, 176, 185, 195, 224, 234, 245, 278, 289, 301 },
        magicLevel = { 3, 5, 7, 10, 12, 14, 16, 18, 20, 23, 24, 25, 28, 29, 30, 33, 34, 35 },
        mpConsume = { 8, 10, 11, 13, 15, 17, 19, 21, 24, 26, 28, 29, 33, 35, 35, 38, 40, 41 },
        mpInitialConsume = { 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 9, 9, 9, 10, 10, 11 },
        power = { 50, 58, 67, 83, 95, 107, 121, 135, 151, 176, 185, 195, 224, 234, 245, 278, 289, 301 },
    },
    other = {
        castRange = 600,
        effectRange = 1100,
        hitTime = 5000,
        reuseDelay = 3000,
        icon = "icon.skill1011",
        isMagic = true,
        targetType = "One",
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
        AbnormalEffects.launch_restore_skill(entity, target_entity, skill_ref, definition, "Hp")
    end,
}
return Skill
