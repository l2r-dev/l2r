require("data.scripts.Utils")
local DaggerBlow = req("data.scripts.runtime.skills.definitions.common.DaggerBlow")

---@type SkillDefinition
local definition = {
    id = 263,
    levels = 37,
    name = "Deadly Blow",
    description =
    "Attacks the target's vital points with 1107 Power added to P. Atk. Requires a dagger. Over-hit and Half-kill are possible.",
    kind = "Active",
    tables = {
        effectPoints = { -183, -188, -193, -198, -203, -208, -213, -218, -223, -228, -233, -237, -242, -247, -252, -257, -261, -266, -271, -275, -279, -284, -288, -292, -296, -300, -304, -307, -311, -314, -317, -320, -323, -326, -328, -331, -333 },
        magicLevel = { 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74 },
        mpConsume = { 33, 34, 35, 35, 36, 37, 38, 39, 40, 41, 42, 43, 45, 45, 46, 47, 48, 49, 50, 52, 53, 54, 55, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68 },
        power = { 1107, 1176, 1249, 1325, 1405, 1488, 1574, 1664, 1757, 1853, 1953, 2057, 2164, 2274, 2388, 2505, 2625, 2748, 2875, 3004, 3136, 3271, 3408, 3548, 3690, 3834, 3980, 4127, 4275, 4425, 4575, 4726, 4878, 5029, 5180, 5330, 5479 },
    },
    other = {
        blowChance = 30,
        castRange = 40,
        coolTime = 720,
        effectRange = 400,
        hitTime = 1080,
        reuseDelay = 3000,
        icon = "icon.skill0263",
        nextActionAttack = true,
        overHit = true,
        targetType = "One",
    },
    conditions = {
        weaponRequired = { "Dagger" },
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
