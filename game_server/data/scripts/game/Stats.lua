---@class Stats
local Stats = {}

-- Stat categories define the order of stats in each category
-- The order MUST match the Rust enum definitions exactly
local stat_categories = {
    Vitals = {
        "Hp", "Mp", "Cp", "MaxHp", "MaxMp", "MaxCp", "HpRegen", "MpRegen", "CpRegen",
        "MaxRecoverableHp", "MaxRecoverableMp", "MaxRecoverableCp", "ManaCharge", "HealEffect"
    },
    Attack = {
        "PAtk", "PvpPAtkBonus", "PSkillPower", "PvpPSkillBonus", "PAtkSpd", "PvePAtkBonus", "PveSkillBonus",
        "PveBowPAtkBonus", "PveBowSkillBonus", "AttackReuse", "PhysicalSkillReuse", "RythmSkillReuse",
        "MAtk", "PvpMAtkBonus", "PveMAtkBonus", "CastSpd", "MagicSkillReuse", "Accuracy", "PAtkRange",
        "PAtkWidth", "PAtkRandom", "MAtkRange", "AttackCountMax", "EffectKind", "SkillMastery", "SkillMasteryRate"
    },
    Defence = {
        "PDef", "PvpPDefBonus", "MDef", "PvpMDefBonus", "ShieldDefence", "ShieldRate", "ShieldAngle",
        "Evasion", "PSkillEvasion", "PvpPSkillBonus", "DefenceCriticalRate", "DefenceCriticalRateAdditional",
        "DefenceCriticalDamage", "DefenceCriticalDamageAdditional", "DamageZoneVulnerability",
        "MovementVulnerability", "CancelVulnerability", "DebuffVulnerability", "BuffVulnerability",
        "FireResistance", "WindResistance", "WaterResistance", "EarthResistance", "HolyResistance",
        "DarkResistance", "MagicSuccessResistance", "DebuffImmunity", "CancelProficiency", "ReflectDamagePercent",
        "ReflectSkillMagic", "ReflectSkillPhysical", "VengeanceSkillMagicDamage", "VengeanceSkillPhysicalDamage",
        "AbsorbDamagePercent", "TransferDamagePercent", "ManaShieldPercent", "TransferDamageToPlayer",
        "AbsorbManaDamagePercent"
    },
    Movement = {
        "Walk", "Run", "Swim", "FastSwim", "Fly", "FastFly", "Fall"
    },
    Critical = {
        "CriticalDamage", "CriticalDamageFront", "CriticalDamageBack", "CriticalDamageSide", "CriticalDamageAdditional",
        "MagicCriticalDamage", "CriticalRate", "CriticalRateFront", "CriticalRateBack", "CriticalRateSide",
        "BlowRate", "MagicCriticalRate", "AttackCancel"
    },
    Primal = {
        "STR", "CON", "DEX", "INT", "WIT", "MEN"
    },
    ElementPower = {
        "Fire", "Water", "Wind", "Earth", "Holy", "Dark"
    },
    MpConsumption = {
        "PhysicalMpConsumeRate", "MagicalMpConsumeRate", "DanceMpConsumeRate", "BowMpConsumeRate", "MpConsume"
    },
    Inventory = {
        "InventoryLimit", "WarehouseLimit", "FreightLimit", "PrivateSellLimit", "PrivateBuyLimit",
        "DwarfRecipeLimit", "CommonRecipeLimit", "WeightCurrent", "WeightLimit", "WeightPenalty"
    },
    Progress = {
        "Exp", "Sp", "VitalityPoints"
    },
    ProgressLevel = {
        "Level", "PrevLevel"
    },
    ProgressRates = {
        "ExpModifier", "SpModifier", "BonusExp", "BonusSp", "VitalityConsumeRate", "MaxSouls",
        "ExpLostByPvp", "ExpLostByMob", "ExpLostByRaid", "DeathPenaltyByPvp", "DeathPenaltyByMob",
        "DeathPenaltyByRaid"
    },
    Other = {
        "FishingExpertise", "Breath", "BreathMax", "MaxBuffSlots", "MaxDebuffSlots", "MaxRhythmSlots"
    }
}

-- Generate index tables from stat_categories (stat name -> index for vec access)
local StatIndexTables = {}
for category, variants in pairs(stat_categories) do
    local indexTable = {}
    for i, variant in ipairs(variants) do
        indexTable[variant] = i
    end
    StatIndexTables[category .. "Stats"] = indexTable
end

-- Gets a stat value from an entity's stats component
---@param stats_entity Entity The entity to get the stat from
---@param stats_type string The type of stats component (e.g., "AttackStats", "VitalsStats")
---@param stat_variant string The specific stat to retrieve (e.g., "PAtk", "Hp")
---@return number The current value of the stat
function Stats.get(stats_entity, stats_type, stat_variant)
    local statsComponent = world.get_component(stats_entity, types[stats_type])
    if not statsComponent then
        error("Entity missing " .. stats_type .. " component")
    end

    -- Get the index table for this stat type
    local indexTable = StatIndexTables[stats_type]
    if not indexTable then
        error("Unknown stat type: " .. stats_type)
    end

    local index = indexTable[stat_variant]
    if index == nil then
        error("Unknown stat variant: " .. stat_variant .. " for type: " .. stats_type)
    end

    -- VitalsStats is a regular struct with `current` field (FloatStats = GenericStats directly)
    -- Other stats are tuple structs wrapping GenericStats, so access via ._1
    if stats_type == "VitalsStats" then
        return statsComponent.current.values[index]
    else
        return statsComponent._1.values[index]
    end
end

-- Sets a stat value on an entity's stats component
---@param stats_entity Entity The entity to modify
---@param stats_type string The type of stats component (e.g., "AttackStats", "VitalsStats")
---@param stat_variant string The specific stat to set (e.g., "PAtk", "Hp")
---@param value number The new value for the stat
function Stats.set(stats_entity, stats_type, stat_variant, value)
    local statsComponent = world.get_component(stats_entity, types[stats_type])
    if not statsComponent then
        error("Entity missing " .. stats_type .. " component")
    end

    local indexTable = StatIndexTables[stats_type]
    if not indexTable then
        error("Unknown stat type: " .. stats_type)
    end

    local index = indexTable[stat_variant]
    if index == nil then
        error("Unknown stat variant: " .. stat_variant .. " for type: " .. stats_type)
    end

    if stats_type == "VitalsStats" then
        statsComponent.current.values[index] = value
        local variantType = string.sub(stats_type, 1, -2)
        local statVariant = construct(types[variantType], { variant = stat_variant })
        statsComponent.change_flags._1:push(statVariant)
    else
        statsComponent._1.values[index] = value
    end
end

-- Applies damage to an entity, properly handling CP absorption before HP damage
---@param damaged_entity Entity The entity receiving damage
---@param damage number The amount of damage to deal
---@return number The total damage dealt
function Stats.apply_damage(damaged_entity, damage)
    -- Get current CP and HP, handle nil values
    local current_cp = Stats.get(damaged_entity, "VitalsStats", "Cp") or 0
    local current_hp = Stats.get(damaged_entity, "VitalsStats", "Hp") or 0

    local remaining_damage = damage
    local new_cp = current_cp
    local new_hp = current_hp

    -- First deal damage to CP if available
    if current_cp > 0 then
        new_cp = current_cp - remaining_damage
        if new_cp < 0 then
            -- CP is depleted, remaining damage goes to HP
            remaining_damage = -new_cp
            new_cp = 0
        else
            -- All damage absorbed by CP
            remaining_damage = 0
        end
    end

    -- Apply remaining damage to HP if any
    if remaining_damage > 0 then
        new_hp = current_hp - remaining_damage
        if new_hp < 0 then
            new_hp = 0
        end
    end

    -- Update both stats
    Stats.set(damaged_entity, "VitalsStats", "Cp", new_cp)
    Stats.set(damaged_entity, "VitalsStats", "Hp", new_hp)

    return damage
end

-- Restores HP, MP, or CP to an entity, capped at maximum values
-- @param healed_entity The entity to heal
-- @param healing The amount to restore
-- @param restore_type The type of restoration: "Hp", "Mp", or "Cp" (defaults to "Hp")
-- @return The amount of healing applied
function Stats.apply_restore(healed_entity, healing, restore_type)
    restore_type = restore_type or "Hp"
    if restore_type ~= "Hp" and restore_type ~= "Cp" and restore_type ~= "Mp" then
        error("Unknown heal type: " .. restore_type)
    end
    local current_value = Stats.get(healed_entity, "VitalsStats", restore_type)
    local max_value = Stats.get(healed_entity, "VitalsStats", "Max" .. restore_type)
    local new_value = current_value + healing
    if new_value > max_value then
        new_value = max_value
    end
    Stats.set(healed_entity, "VitalsStats", restore_type, new_value)
    return healing
end

-- Consumes MP from an entity if sufficient MP is available
-- @param consume_entity The entity to consume MP from
-- @param mp The amount of MP to consume
-- @return true if MP was successfully consumed, false if insufficient MP
function Stats.consume_mp(consume_entity, mp)
    local current_mp = Stats.get(consume_entity, "VitalsStats", "Mp")
    if current_mp < mp then
        return false
    end
    local new_mp = current_mp - mp
    Stats.set(consume_entity, "VitalsStats", "Mp", new_mp)
    return true
end

-- Checks if an entity has sufficient MP without consuming it
-- @param consume_entity The entity to check
-- @param mp The amount of MP to check for
-- @return true if the entity has sufficient MP, false otherwise
function Stats.check_mp(consume_entity, mp)
    local current_mp = Stats.get(consume_entity, "VitalsStats", "Mp")
    if current_mp < mp then
        return false
    end
    return true
end

local stat_categories = {
    Vitals = {
        "Hp", "Mp", "Cp", "MaxHp", "MaxMp", "MaxCp", "HpRegen", "MpRegen", "CpRegen",
        "MaxRecoverableHp", "MaxRecoverableMp", "MaxRecoverableCp", "ManaCharge", "HealEffect"
    },
    Attack = {
        "PAtk", "PvpPAtkBonus", "PSkillPower", "PvpPSkillBonus", "PAtkSpd", "PvePAtkBonus", "PveSkillBonus",
        "PveBowPAtkBonus", "PveBowSkillBonus", "AttackReuse", "PhysicalSkillReuse", "RythmSkillReuse",
        "MAtk", "PvpMAtkBonus", "PveMAtkBonus", "CastSpd", "MagicSkillReuse", "Accuracy", "PAtkRange",
        "PAtkWidth", "PAtkRandom", "MAtkRange", "PAtkMaxTargetsCount", "EffectKind"
    },
    Defence = {
        "PDef", "PvpPDefBonus", "MDef", "PvpMDefBonus", "ShieldDefence", "ShieldRate", "ShieldAngle",
        "Evasion", "PSkillEvasion", "PvpPSkillBonus", "DefenceCriticalRate", "DefenceCriticalRateAdditional",
        "DefenceCriticalDamage", "DefenceCriticalDamageAdditional", "DamageZoneVulnerability",
        "MovementVulnerability", "CancelVulnerability", "DebuffVulnerability", "BuffVulnerability",
        "FireResistance", "WindResistance", "WaterResistance", "EarthResistance", "HolyResistance",
        "DarkResistance", "MagicSuccessResistance", "DebuffImmunity"
    },
    Movement = {
        "Walk", "Run", "Swim", "FastSwim", "Fly", "FastFly", "Fall"
    },
    Critical = {
        "CriticalDamage", "CriticalDamageFront", "CriticalDamageBack", "CriticalDamageSide", "CriticalDamageAdditional",
        "MagicCriticalDamage", "CriticalRate", "CriticalRateFront", "CriticalRateBack", "CriticalRateSide",
        "BlowRate", "MagicCriticalRate", "AttackCancel"
    },
    Primal = {
        "STR", "CON", "DEX", "INT", "WIT", "MEN"
    },
    ElementPower = {
        "Fire", "Water", "Wind", "Earth", "Holy", "Dark"
    },
    ReflectAbsorb = {
        "CancelProficiency", "ReflectDamagePercent", "ReflectSkillMagic", "ReflectSkillPhysical",
        "VengeanceSkillMagicDamage", "VengeanceSkillPhysicalDamage", "AbsorbDamagePercent",
        "TransferDamagePercent", "ManaShieldPercent", "TransferDamageToPlayer", "AbsorbManaDamagePercent"
    },
    Inventory = {
        "InventoryLimit", "WarehouseLimit", "FreightLimit", "PrivateSellLimit", "PrivateBuyLimit",
        "DwarfRecipeLimit", "CommonRecipeLimit", "WeightCurrent", "WeightLimit", "WeightPenalty"
    },
    MpConsumption = {
        "PhysicalMpConsumeRate", "MagicalMpConsumeRate", "DanceMpConsumeRate", "BowMpConsumeRate", "MpConsume"
    },
    SkillMastery = {
        "SkillMastery", "SkillMasteryRate"
    },
    Progress = {
        "Exp", "Sp", "VitalityPoints"
    },
    ProgressLevel = {
        "Level", "PrevLevel"
    },
    ProgressRates = {
        "ExpModifier", "SpModifier", "ExpSpRate", "BonusExp", "BonusSp",
        "VitalityConsumeRate", "MaxSouls", "ExpLostByPvp", "ExpLostByMob", "ExpLostByRaid",
        "DeathPenaltyByPvp", "DeathPenaltyByMob", "DeathPenaltyByRaid"
    },
    Other = {
        "FishingExpertise", "Breath", "BreathMax"
    }
}

-- Precompute the mapping of stat variants to their types
local variant_to_type = {}
for category, variants in pairs(stat_categories) do
    for _, variant in ipairs(variants) do
        variant_to_type[variant] = category
    end
end

-- Creates a StatKind enum from a stat variant string
-- @param stat_variant The stat variant name (e.g., "PAtk", "Hp", "STR")
-- @return A StatKind object with the appropriate category and variant
function Stats.kind(stat_variant)
    local stat_type = variant_to_type[stat_variant]
    if not stat_type then
        error("Unknown stat variant: " .. stat_variant)
    end

    -- Construct the appropriate stat variant object
    local stat_variant_object = construct(types[stat_type .. "Stat"], { variant = stat_variant })

    -- Construct and return a StatKind object
    return construct(types.StatKind, { variant = stat_type, _1 = stat_variant_object })
end

-- Calculates the actual casting time for a skill based on attacker's attack speed
-- This adjusts the base hit time from skill definition based on PAtkSpd or CastSpd
-- @param attacker_entity The entity casting the skill
-- @param skill_definition The skill definition containing isMagic flag and hitTime
-- @return The adjusted casting time in milliseconds (0 for instant skills)
function Stats.calc_cast_time(attacker_entity, skill_definition)
    local base_hit_time = skill_definition.other.hitTime

    -- Some skills (like toggles) have no cast time and are instant
    if base_hit_time == nil or base_hit_time == 0 then
        return 0
    end

    if skill_definition.other.isMagic then
        -- Magic skills use CastSpd (base 333)
        local cast_spd = Stats.get(attacker_entity, "AttackStats", "CastSpd")
        return math.floor((base_hit_time / cast_spd) * 333)
    else
        -- Physical skills use PAtkSpd (base 300)
        local patk_spd = Stats.get(attacker_entity, "AttackStats", "PAtkSpd")
        return math.floor((base_hit_time / patk_spd) * 300)
    end
end

-- Calculates the actual reuse delay (cooldown) for a skill based on reuse rate stats
---@param attacker_entity Entity entity using the skill
---@param skill_definition SkillDefinition skill definition containing reuseDelay and skill type flags
---@return number delay reuse in milliseconds
function Stats.calc_reuse_delay(attacker_entity, skill_definition)
    local base_reuse = skill_definition.other.reuseDelay

    -- Skills with no reuse delay
    if base_reuse == nil or base_reuse == 0 then
        return 0
    end

    -- Static reuse skills don't get modified by stats (isStaticReuse or isStatic flags)
    if skill_definition.other.isStaticReuse or skill_definition.other.isStatic then
        return base_reuse
    end

    -- Get the appropriate reuse rate modifier based on skill type
    -- Default is 1.0 (100% - no modification)
    local reuse_rate = 1.0
    if skill_definition.other.isMagic then
        -- Magic skills use MagicSkillReuse stat
        reuse_rate = Stats.get(attacker_entity, "AttackStats", "MagicSkillReuse")
    elseif skill_definition.other.isPhysical then
        -- Physical skills use PhysicalSkillReuse stat
        reuse_rate = Stats.get(attacker_entity, "AttackStats", "PhysicalSkillReuse")
    elseif skill_definition.other.isRythm then
        -- Rythm skills use RythmSkillReuse stat
        reuse_rate = Stats.get(attacker_entity, "AttackStats", "RythmSkillReuse")
    end

    -- Apply the reuse rate multiplier
    -- Examples: 1.0 = no change, 0.85 = -15% reuse, 0.7 = -30% reuse, 3.0 = +200% reuse
    return math.floor(base_reuse * reuse_rate)
end

return Stats
