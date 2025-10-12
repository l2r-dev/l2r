---@class SkillStatModifier Skill-level stat modifier (tuple format for skill definitions)
---@field [1] StatOp Operation type
---@field [2] number[] Values per skill level

---@class SkillConditionalStatModifier
---@field [1] SkillStatModifier The stat modifier
---@field condition table Condition that must be met (e.g., { armor = "Light" })

---@class SkillStatsTables
---@field PAtk? SkillStatModifier Physical Attack modifier
---@field PDef? SkillStatModifier Physical Defense modifier
---@field MAtk? SkillStatModifier Magical Attack modifier
---@field MDef? SkillStatModifier Magical Defense modifier
---@field Evasion? SkillStatModifier Evasion modifier
---@field Accuracy? SkillStatModifier Accuracy modifier
---@field CritRate? SkillStatModifier Critical Rate modifier
---@field MpRegen? SkillStatModifier MP Regeneration modifier
---@field HpRegen? SkillStatModifier HP Regeneration modifier
---@field Speed? SkillStatModifier Movement Speed modifier
---@field AtkSpeed? SkillStatModifier Attack Speed modifier
---@field CastSpeed? SkillStatModifier Casting Speed modifier

---@class SkillOptionalStatsTables
---@field [string]? SkillConditionalStatModifier Any other conditional stat modifier

---@class SkillOptionalStatsGroup A group of conditional stats with a shared condition
---@field condition table Condition that must be met (e.g., { armor = "Light" }, { character_level_le = 9 })
---@field stats table<string, SkillStatModifier> Stats to apply when condition is met

---@class SkillLevelTables
---@field abnormalLevels? number[] Abnormal effect levels per skill level
---@field effectPoints? number[] Effect point values per skill level
---@field magicLevel? number[] Magic level requirements per skill level
---@field mpConsume? number[] MP consumption per skill level
---@field mpInitialConsume? number[] Initial MP consumption per skill level
---@field hpConsume? number[] HP consumption per skill level
---@field power? number[] Skill power per skill level
---@field stats? SkillStatsTables Stat modifiers for this skill
---@field optional_stats? SkillOptionalStatsGroup[] Array of conditional stat groups, each with condition and stats

---@class SkillConditions
---@field weaponRequired? string[] Required weapon types (e.g., {"Dagger"})
---@field armorRequired? string[] Required armor types
---@field [string]? any Other custom conditions

---@class SkillOtherProperties
---@field abnormalTime? number Duration of abnormal effect in milliseconds
---@field abnormalKind? string Type of abnormal effect (e.g., "PaUp", "PdUp")
---@field blowChance? number Chance for blow attacks
---@field castRange? number Casting range in units
---@field coolTime? number Cool down time in milliseconds
---@field effectRange? number Effect range in units
---@field hitTime? number Time to hit in milliseconds
---@field reuseDelay? number Reuse delay in milliseconds
---@field icon? string Icon resource path
---@field isMagic? boolean Whether skill is magical
---@field isPhysical? boolean Whether skill is physical
---@field isRythm? boolean Whether skill is a dance or song
---@field nextActionAttack? boolean Whether next action is an attack
---@field overHit? boolean Whether over-hit is possible
---@field targetType? string Target type ("Self", "One", "Area", etc.)
---@field priority? number Skill priority level
---@field [string]? any Other custom properties

---@class SkillKindReference ReflectReference to skills::Kind enum
---@field variant string The skill kind variant name: "Active", "Passive", "Toggle"

---@alias SkillKind "Active"|"Passive"|"Toggle"

---@class SkillDefinition
---@field id number Unique skill ID
---@field levels number Number of skill levels
---@field name string Skill name
---@field description string Skill description
---@field kind SkillKind Skill kind
---@field tables SkillLevelTables Level-based tables for skill parameters
---@field other SkillOtherProperties Other skill properties and metadata
---@field conditions? SkillConditions Skill usage conditions (optional)

---@class SkillHandler
---@field definition SkillDefinition Skill definition containing all skill configuration
---@field pend? fun(entity: Entity, skill_ref: Skill, shift_pressed: boolean, ctrl_pressed: boolean): nil
---@field on_pending? fun(entity: Entity, pending_skill: LuaPendingSkillData): nil Handles pending skill processing
---@field launch? fun(entity: Entity, target_entity: Entity, skill_ref: Skill): nil Launches the skill against a target (or self when target_entity is nil)
---@field apply_abnormal? fun(target_entity: Entity, skill_ref: Skill): nil Applies abnormal/state effects when an abnormal is created
---@field apply_passive? fun(target_entity: Entity, skill_ref: Skill): nil Applies passive skill modifiers (for passive skills)
