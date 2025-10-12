require("data.scripts.Utils")
local PassiveSkills = req("data.scripts.game.PassiveSkills")
local PaperDoll = req("data.scripts.game.PaperDoll")

---@type SkillDefinition
local definition = {
    id = 163,
    levels = 1,
    name = "Spellcraft",
    description = "Increases Casting Spd. when equipped with a robe jacket and robe pants.",
    kind = "Passive",
    tables = {
        magicLevel = { 1 },
        optional_stats = {
            {
                condition = { armor = "Magic" },
                stats = {
                    CastSpd = { "Mul", { 1.5 } },
                },
            },
        },
    },
    other = { icon = "icon.skill0163" },
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
