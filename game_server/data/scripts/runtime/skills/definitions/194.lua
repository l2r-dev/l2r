require("data.scripts.Utils")
local PassiveSkills = req("data.scripts.game.PassiveSkills")
local Logger = req("data.scripts.Logger")
local Stats = req("data.scripts.game.Stats")

---@type SkillDefinition
local definition = {
    id = 194,
    levels = 1,
    name = "Lucky",
    description = "Decrease exp loss ",
    kind = "Passive",
    tables = {
        magicLevel = { 9 },
        optional_stats = {
            {
                condition = { character_level_le = 9 },
                stats = {
                    ExpLostByMob = { "Set", { 0.0 } },
                    ExpLostByPvp = { "Set", { 0.0 } },
                    ExpLostByRaid = { "Set", { 0.0 } },
                }
            }
        },
    },
    other = { icon = "icon.skill0142" },
}


---@type SkillHandler
local Skill = {
    definition = definition,
    apply_passive = function(entity, skill_ref)
        local level = skill_ref.level._1
        local required_character_level = definition.tables.optional_stats[1].condition.character_level_le
        local char_level = Stats.get(entity, "ProgressLevelStats", "Level")
        local is_proper_level = char_level <= required_character_level

        for stat_name, operation_data in pairs(definition.tables.optional_stats[1].stats) do
            PassiveSkills.apply_conditional_stat_modifier(
                entity,
                definition,
                level,
                stat_name,
                operation_data,
                is_proper_level
            )
        end
    end,
}
return Skill
