require("data.scripts.Utils")
local Magic = req("data.scripts.runtime.skills.definitions.common.Magic")
local Target = req("data.scripts.runtime.skills.definitions.common.Target")

---@type SkillDefinition
local definition = {
    id = 1177,
    levels = 5,
    name = "Wind Strike",
    description =
    "Supplements the user's M. Atk. with 12 Power to create a powerful hurricane that inflicts Wind damage on the enemy.",
    kind = "Active",
    tables = {
        effectPoints = { -92, -106, -121, -143, -162 },
        magicLevel = { 1, 4, 7, 11, 14 },
        mpConsume = { 7, 7, 8, 11, 12 },
        mpInitialConsume = { 2, 2, 2, 3, 3 },
        power = { 12, 13, 15, 18, 21 },
    },
    other = {
        castRange = 600,
        effectRange = 1100,
        element = "Wind",
        elementPower = 20,
        hitTime = 4000,
        reuseDelay = 1000,
        icon = "icon.skill1177",
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
