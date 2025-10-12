require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")

---@type SkillDefinition
local definition = {
    id = 1239,
    levels = 28,
    name = "Hurricane",
    description = "Creates a whirlwind that uses Power to inflict Wind damage to the enemy.",
    kind = "Active",
    tables = {
        effectPoints = { -360, -379, -399, -418, -438, -457, -477, -495, -514, -532, -541, -549, -558, -566, -574, -582, -590, -597, -604, -611, -617, -624, -630, -635, -641, -646, -650, -655 },
        magicLevel = { 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74 },
        mpConsume = { 27, 28, 30, 31, 33, 35, 36, 38, 39, 41, 42, 43, 44, 44, 45, 46, 47, 48, 48, 49, 50, 51, 51, 52, 53, 53, 54, 55 },
        mpInitialConsume = { 7, 7, 8, 8, 9, 9, 9, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14 },
        power = { 49, 52, 55, 58, 61, 65, 68, 72, 75, 78, 80, 82, 84, 85, 87, 89, 90, 92, 94, 96, 97, 99, 100, 102, 104, 105, 107, 108 },
    },
    other = {
        castRange = 900,
        effectRange = 1400,
        element = "Wind",
        elementPower = 20,
        hitTime = 4000,
        reuseDelay = 1200,
        icon = "icon.skill1239",
        isMagic = true,
        targetType = "One",
    },
}

---@type SkillHandler
local Skill = {
    definition = definition,

    pend = function(entity, skill_ref, shift_pressed, ctrl_pressed)
        Target.pend_skill_on_enemy(entity, skill_ref, definition, ctrl_pressed, shift_pressed)
    end,
    on_pending = function(entity, pending_skill)
        Magic.on_pending_skill(entity, pending_skill, definition)
    end,
    launch = function(entity, target_entity, skill_ref)
        Magic.launch_magical_bolt_skill(entity, target_entity, skill_ref, definition)
    end,
}

return Skill
