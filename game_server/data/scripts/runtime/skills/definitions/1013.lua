require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")
local AbnormalEffects = req("data.scripts.game.AbnormalEffects")

---@type SkillDefinition
local definition = {
    id = 1013,
    levels = 32,
    name = "Recharge",
    description =
    "Recovers MP with up to 49 Power, depending on the target's level. Cannot be used by classes that have Recharge skill.",
    kind = "Active",
    tables = {
        power = { 49, 52, 57, 60, 66, 70, 73, 77, 81, 86, 90, 94, 98, 102, 104, 106, 108, 110, 113, 115, 116, 118, 120, 122, 124, 126, 128, 129, 131, 133, 134, 136 },
        effectPoints = { 268, 285, 313, 331, 360, 379, 399, 418, 438, 457, 477, 495, 514, 532, 541, 549, 558, 566, 574, 582, 590, 597, 604, 611, 617, 624, 630, 635, 641, 646, 650, 655 },
        magicLevel = { 28, 30, 33, 35, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74 },
        mpConsume = { 39, 42, 45, 48, 53, 56, 59, 62, 65, 69, 72, 75, 78, 82, 83, 85, 87, 88, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104, 105, 106, 108, 109 },
        mpInitialConsume = { 10, 11, 12, 12, 14, 14, 15, 16, 17, 18, 18, 19, 20, 21, 21, 22, 22, 22, 23, 23, 24, 24, 24, 25, 25, 26, 26, 26, 27, 27, 27, 28 },
    },
    other = {
        castRange = 400,
        effectRange = 900,
        hitTime = 6000,
        reuseDelay = 3000,
        icon = "icon.skill1013",
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
        AbnormalEffects.launch_restore_skill(entity, target_entity, skill_ref, definition, "Mp")
    end,
}
return Skill
