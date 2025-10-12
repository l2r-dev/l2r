require("data.scripts.Utils")
local PassiveSkills = req("data.scripts.game.PassiveSkills")
local PaperDoll = req("data.scripts.game.PaperDoll")

---@type SkillDefinition
local definition = {
    id = 142,
    levels = 5,
    name = "Armor Mastery",
    description = "Increases P. Def. by 9 and MP Recovery Bonus by 10%.",
    kind = "Passive",
    tables = {
        magicLevel = { 5, 8, 10, 13, 15 },
        stats = {
            PDef = { "Add", { 9, 11, 12, 13, 14 } },
            MpRegen = { "Mul", { 1.1, 1.1, 1.1, 1.1, 1.1 } },
        },
        optional_stats = {
            {
                condition = { armor = "Light" },
                stats = {
                    Evasion = { "Add", { 0, 0, 0, 3, 3 } },
                },
            },
        },
    },
    other = { icon = "icon.skill0142" },
}


---@type SkillHandler
local Skill = {
    definition = definition,
    apply_passive = function(entity, skill_ref)
        local level = skill_ref.level._1
        local armor_kind = definition.tables.optional_stats[1].condition.armor
        local has_proper_armor = PaperDoll.has_required_armor(entity, armor_kind)

        for stat_name, operation_data in pairs(definition.tables.optional_stats[1].stats) do
            PassiveSkills.apply_conditional_stat_modifier(
                entity,
                definition,
                level,
                stat_name,
                operation_data,
                has_proper_armor
            )
        end

        PassiveSkills.apply_stat_modifiers(entity, definition, level)
    end,
}
return Skill
