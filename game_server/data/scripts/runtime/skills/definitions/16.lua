require("data.scripts.Utils")
local DaggerBlow = req("data.scripts.runtime.skills.definitions.common.DaggerBlow")

---@type SkillDefinition
local definition = {
    id = 16,
    levels = 24,
    name = "Mortal Blow",
    description =
    "Attacks the target's vital points with 73 Power added to P. Atk. Requires a dagger. Over-hit is possible. Cannot be used with Dual Daggers.",
    kind = "Active",
    tables = {
        effectPoints = { -52, -54, -57, -64, -67, -70, -79, -83, -86, -96, -100, -104, -111, -115, -119, -128, -132, -136, -145, -150, -154, -164, -169, -173 },
        magicLevel = { 3, 4, 5, 8, 9, 10, 13, 14, 15, 18, 19, 20, 22, 23, 24, 26, 27, 28, 30, 31, 32, 34, 35, 36 },
        mpConsume = { 8, 8, 9, 10, 11, 11, 14, 15, 15, 18, 18, 18, 19, 20, 21, 22, 23, 24, 26, 26, 26, 28, 29, 30 },
        power = { 73, 80, 88, 115, 126, 137, 178, 193, 210, 268, 291, 314, 367, 396, 427, 494, 531, 571, 656, 703, 752, 859, 916, 977 },
    },
    other = {
        isPhysical = true,
        critRate = 20,
        blowChance = 20,
        castRange = 40,
        coolTime = 720,
        effectRange = 400,
        hitTime = 1080,
        reuseDelay = 3000,
        icon = "icon.skill0016",
        nextActionAttack = true,
        overHit = true,
        targetType = "One",
    },
    conditions = {
        weaponRequired = { "Dagger" },
        weaponExtraKind = { "OneHanded" },
    },
}

---@type SkillHandler
local Skill = {
    definition = definition,
    pend = function(entity, skill_ref, shift_pressed, ctrl_pressed)
        DaggerBlow.pend(entity, skill_ref, definition, shift_pressed, ctrl_pressed)
    end,
    on_pending = function(entity, pending_skill)
        DaggerBlow.handle_pending_blow(entity, pending_skill, definition)
    end,
    launch = function(entity, target_entity, skill_ref)
        DaggerBlow.launch_skill(entity, target_entity, skill_ref, definition)
    end,
}
return Skill
