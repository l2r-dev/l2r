local Stats = req("data.scripts.game.Stats")
local StatModifiers = req("data.scripts.game.StatModifiers")
local SystemMessage = req("data.scripts.packets.SystemMessage")
local Logger = req("data.scripts.Logger")
Logger.set_script_name("game.AbnormalEffects")

---@class AbnormalEffects
local AbnormalEffects = {}

local abnormal_categories = {
    Rhythm = {
        -- Dance
        "DanceDefenceMotion1", "DanceOfAlignment", "DanceOfAquaGuard", "DanceOfBerserker",
        "DanceOfBladestorm", "DanceOfConcentration", "DanceOfEarthGuard", "DanceOfFire",
        "DanceOfFury", "DanceOfInspiration", "DanceOfLight", "DanceOfMystic",
        "DanceOfProtection", "DanceOfShadow", "DanceOfSiren", "DanceOfVampire", "DanceOfWarrior",
        -- Song
        "SongBattleWhisper", "SongOfChampion", "SongOfEarth", "SongOfElemental",
        "SongOfFlameGuard", "SongOfHunter", "SongOfInvocation", "SongOfLife",
        "SongOfMeditation", "SongOfPurification", "SongOfRenewal", "SongOfStormGuard",
        "SongOfVengeance", "SongOfVitality", "SongOfWarding", "SongOfWater",
        "SongOfWind", "SongOfWindstorm"
    },
    Buff = {
        "AbilityChange", "AllRegenUp", "ArmorEarth", "ArmorFire", "ArmorHoly", "ArmorUnholy",
        "ArmorWater", "ArmorWind", "AttackSpeedUpBow", "AttackTimeUp", "AttributePotion",
        "AvoidUp", "AvoidUpSpecial", "Berserker", "BlessTheBlood", "BlockShieldUp",
        "BlockSpeedUp", "BuffQueenOfCat", "BuffUnicornSeraphim", "CastingTimeDown",
        "CheapMagic", "Combination", "CounterCritical", "CounterSkill", "CpUp", "CriticalRateUp",
        "CriticalDmgUp", "CriticalDmgUpToggle", "CriticalProbUp", "DmgShield", "DragonBuff",
        "DuelistSpirit", "DwarfAttackBuff", "DwarfDefenceBuff", "ElementalArmor", "EvasionBuff",
        "HealEffectUp", "HealPowerUp", "HeroBuff", "Hide", "HitUp", "HolyAttack",
        "HpRecover", "HpRegenUp", "ImproveCrtRateDmgUp", "ImproveHitDefenceCrtRateUp",
        "ImproveHpMpUp", "ImproveMaMdUp", "ImprovePaPdUp", "ImproveShieldRateDefenceUp",
        "ImproveSpeedAvoidUp", "ImproveVampiricHaste", "Instinct", "Invincibility",
        "IronShield", "IronShieldI", "KnightAura", "LifeForceKamael", "LifeForceOrc",
        "LifeForceOthers", "MagicalStance", "MagicCriticalUp", "Majesty", "MaximumAbility",
        "MaxBreathUp", "MaxHpUp", "MaxHpCpUp", "MaxMpUp", "MaMdUp", "MaUp", "MaUpHerb",
        "MaUpSpecial", "MdUp", "MdUpAttr", "MightMortal", "Mirage", "MpCostDown",
        "MpRegenUp", "MultiBuff", "MultiBuffA", "Patience", "PaPdUp", "PaUp",
        "PaUpHerb", "PaUpSpecial", "PdUp", "PdUpBow", "PdUpSpecial", "PhysicalStance",
        "PkProtect", "PotionOfGenesis", "PreserveAbnormal", "Protection", "PvpWeaponBuff",
        "RageMight", "RechargeUp", "ReduceDropPenalty", "ReflectAbnormal", "ReflectMagicDd",
        "ResistBleeding", "ResistDebuffDispel", "ResistDerangement", "ResistHolyUnholy",
        "ResistPoison", "ResistShock", "ResistSpiritless", "ResurrectionSpecial",
        "ReuseDelayDown", "SeedOfCritical", "SeedOfKnight", "ShieldDefenceUp",
        "ShieldProbUp", "SkillIgnore", "Snipe", "SpeedUp", "SpeedUpSpecial",
        "SsqTownBlessing", "Stealth", "SubTriggerCrtRateUp", "SubTriggerDefence",
        "SubTriggerHaste", "SubTriggerSpirit", "Talisman", "TCrtDmgUp", "TCrtRateUp",
        "ThrillFight", "TouchOfLife", "UltimateBuff", "ValakasItem", "VampiricAttack",
        "VampiricAttackSpecial", "Vote", "VpKeep", "VpUp", "WeaponMastery", "Will",
        "AbnormalInvincibility", "AbnormalItem", "AbHawkEye", "Apella", "ArcherSpecial",
        "ArcherSpecialI", "ArrowRain", "AvoidSkill", "BigBody", "BigHead", "BloodContract",
        "BrEventBuf1", "BrEventBuf10", "BrEventBuf2", "BrEventBuf3", "BrEventBuf5",
        "BrEventBuf6", "BrEventBuf7", "CounterCriticalTrigger", "DamageAmplify", "DdResist",
        "Deathworm", "EntryForGame", "EventGawi", "EventSantaReward", "EventTerritory",
        "EventWin", "FinalSecret", "FlameHawk", "FlyAway", "FocusDagger", "ForceMeditation",
        "ForceOfDestruction", "GhostPiercing", "HotGround", "KamaelSpecial", "Limit",
        "None", "NormalAttackBlock", "PolearmAttack", "PublicSlot", "RealTarget",
        "SignalA", "SignalB", "SignalC", "SignalD", "SignalE", "SoaBuff1", "SoaBuff2",
        "SoaBuff3", "SummonCondition", "SuperHasteToggle", "TimeCheck", "WpChangeEvent"
    },
    Debuff = {
        "AllAttackDown", "AllSpeedDown", "Anesthesia", "AntarasDebuff", "AttackTimeDown",
        "AttackTimeDownSpecial", "AvoidDown", "BetrayalMark", "Bleeding", "BlockResurrection",
        "BlockTransform", "BotPenalty", "CancelProbDown", "CastingTimeUp", "CriticalDmgDown",
        "CriticalPoison", "CriticalProbDown", "CurseLifeFlow", "DarkSeed", "DeathClack",
        "DeathMark", "DeathPenalty", "DebuffNightshade", "DebuffShield", "DecreaseWeightPenalty",
        "Derangement", "DetectWeakness", "Disarm", "DotAttr", "DotMp", "DragonBreath",
        "Enervation", "EvilBlood", "ExposeWeakPoint", "FatalPoison", "FireDot",
        "FishingMasteryDown", "Freezing", "HealEffectDown", "HeroDebuff", "HitDown",
        "HpRegenDown", "MaDown", "MaxHpDown", "MdDown", "MentalImpoverish", "Meteor",
        "MirageTrap", "MpCostUp", "MultiDebuff", "MultiDebuffA", "MultiDebuffB",
        "MultiDebuffC", "MultiDebuffD", "MultiDebuffE", "MultiDebuffF", "MultiDebuffFire",
        "MultiDebuffG", "MultiDebuffHoly", "MultiDebuffSoul", "MultiDebuffUnholy",
        "MultiDebuffWater", "MultiDebuffWind", "Oblivion", "Paralyze", "PaDown",
        "PdDown", "Pinch", "Poison", "Possession", "PossessionSpecial", "PvpDmgDown",
        "PvpWeaponDebuff", "ReuseDelayUp", "RootMagically", "RootPhysically", "SeizureA",
        "SeizureB", "SeizureC", "SeizurePenalty", "Silence", "SilenceAll",
        "SilencePhysical", "Sleep", "SpaDiseaseA", "SpaDiseaseB", "SpaDiseaseC",
        "SpaDiseaseD", "SpeedDown", "Spite", "SpoilBomb", "SsqTownCurse", "StarFall",
        "StigmaA", "StigmaOfSilen", "Stun", "TargetLock", "ThinSkin", "TouchOfDeath",
        "TransferDamage", "Transform", "TransformHangover", "TransformScrifice",
        "TransformScrificeP", "TurnFlee", "TurnPassive", "TurnStone", "TCrtDmgDown",
        "UltimateDebuff", "Vibration", "WatcherGaze", "WaterDot", "WeakConstitution", "WindDot"
    },
}

local last_change_ticks = {}

-- Check if an entity's abnormal effects have changed since the last check
---@param entity any The entity to check
---@return boolean True if abnormal effects have changed
function AbnormalEffects.is_changed(entity)
    local entity_key = tostring(entity:index())
    local component_ticks = world.get_component_ticks(entity, types.AbnormalEffectsChangeTracker)

    local last_tick = last_change_ticks[entity_key]
    if not last_tick then
        last_tick = construct(types.Tick, { tick = 0 })
        last_change_ticks[entity_key] = last_tick
    end

    local is_changed = component_ticks:is_changed(last_tick, component_ticks.changed)

    if is_changed then
        last_change_ticks[entity_key] = component_ticks.changed
    end

    return is_changed
end

-- Collect all active effects from an entity
---@param abnormal_effects any AbnormalEffects component
---@return table Array of active effects
local function collect_active_effects(abnormal_effects)
    local effects = {}

    if not abnormal_effects or not abnormal_effects.effects then
        return effects
    end

    for _, effects_vec in pairs(abnormal_effects.effects) do
        if effects_vec then
            for i = 1, #effects_vec do
                local effect = effects_vec[i]
                if effect and effect.active then
                    table.insert(effects, effect)
                end
            end
        end
    end

    return effects
end

-- Send AbnormalStatusUpdate packet to client with all active effects
---@param entity any The entity to send the packet to
local function send_abnormal_status_update(entity)
    local abnormal_effects = world.get_component(entity, types.AbnormalEffects)
    local timers = world.get_component(entity, types.AbnormalEffectsTimers)
    local effects = collect_active_effects(abnormal_effects)

    local durations = {}
    for _, effect in ipairs(effects) do
        -- Get remaining time from timers, -1 for infinite/toggle effects
        local remaining_time = -1
        if timers then
            local timer_data = timers._1[effect.skill.id]
            if timer_data and timer_data.timer then
                remaining_time = timer_data.timer:remaining():as_secs()
            end
        end
        table.insert(durations, remaining_time)
    end

    local abnormal_status_update = construct(types.AbnormalStatusUpdate, {
        effects = effects,
        durations = durations
    })

    local packet = construct(types.GameServerPacket, {
        variant = "AbnormalStatusUpdate",
        _1 = abnormal_status_update
    })

    GameServerPacket.send({ entity, packet })
end

-- Precompute the mapping of abnormal effect variants to their types
local variant_to_type = {}
for category, variants in pairs(abnormal_categories) do
    for _, variant in ipairs(variants) do
        variant_to_type[variant] = category
    end
end

-- Creates an AbnormalKind enum from an abnormal effect variant string
---@param effect_variant string The abnormal effect variant (e.g., "Poison", "Bleeding", "DanceOfFire")
---@return any An AbnormalKind object with the appropriate category and variant
function AbnormalEffects.kind(effect_variant)
    local effect_kind = variant_to_type[effect_variant]
    if not effect_kind then
        error("Unknown abnormal effect variant: " .. effect_variant)
    end

    local effect_variant_object = construct(types[effect_kind .. "Kind"], { variant = effect_variant })
    return construct(types.AbnormalKind, { variant = effect_kind, _1 = effect_variant_object })
end

-- Creates an AbnormalEffect struct with duration (timers are managed separately)
---@param skill any The skill that applies this abnormal effect
---@param duration_ms number The duration in milliseconds for the effect
---@param kind_variant string The AbnormalKind variant string (e.g., "Poison", "Bleeding", etc.)
---@param effects_over_time table Array of EffectOverTime objects
---@return any, any Returns both the effect and timer data
function AbnormalEffects.effect(skill, duration_ms, kind_variant, effects_over_time)
    local abnormal_kind = AbnormalEffects.kind(kind_variant)

    local effect = construct(types.AbnormalEffect, {
        skill = skill,
        active = true,
        kind = abnormal_kind,
    })

    local timer_mode_variant = construct(types.TimerMode, { variant = "Once" })
    local duration = Duration.from_millis(duration_ms)
    local timer = Timer.new(duration, timer_mode_variant)

    local timer_data = construct(types.AbnormalEffectTimer, {
        timer = timer,
        effects_over_time = effects_over_time,
    })

    return effect, timer_data
end

-- Creates an infinite duration abnormal effect (no timer)
---@param skill any The skill that applies this abnormal effect
---@param kind_variant string The AbnormalKind variant string
---@param effects_over_time table Array of EffectOverTime objects (optional)
---@return any, any|nil Returns the effect and timer data (timer will be nil for infinite)
function AbnormalEffects.infinite_effect(skill, kind_variant, effects_over_time)
    local abnormal_kind = AbnormalEffects.kind(kind_variant)

    local effect = construct(types.AbnormalEffect, {
        skill = skill,
        active = true,
        kind = abnormal_kind,
    })

    local timer_data = nil
    if effects_over_time and #effects_over_time > 0 then
        timer_data = construct(types.AbnormalEffectTimer, {
            timer = nil,
            effects_over_time = effects_over_time,
        })
    end

    return effect, timer_data
end

-- Applies an abnormal effect to a target entity
---@param target_entity any The entity to apply the effect to
---@param effect any The AbnormalEffect to apply
---@param timer_data any|nil The timer data (optional)
---@param skill_definition table The skill definition for system messages
function AbnormalEffects.apply_effect(target_entity, effect, timer_data, skill_definition)
    if not target_entity or not effect then
        return
    end

    AbnormalEffect.add({ target_entity, effect })

    if timer_data then
        local timers = world.get_component(target_entity, types.AbnormalEffectsTimers)
        if timers then
            timers._1[effect.skill.id] = timer_data
        end
    end
    SystemMessage.send_with_text(target_entity, 110, skill_definition.name)
    send_abnormal_status_update(target_entity)
end

-- Removes an abnormal effect from a target entity by skill ID
---@param target_entity any The entity to remove the effect from
---@param skill_id any The ID of the skill effect to remove
---@param skill_definition table The skill definition for system messages
function AbnormalEffects.remove_effect(target_entity, skill_id, skill_definition)
    if not target_entity or not skill_id then
        return
    end

    if skill_definition.other and skill_definition.other.abnormalKind then
        AbnormalEffects.remove_stat_modifiers(target_entity, skill_definition.other.abnormalKind)
    end
    AbnormalEffect.remove({ target_entity, skill_id })
    SystemMessage.send_with_text(target_entity, 749, skill_definition.name)
    send_abnormal_status_update(target_entity)
end

-- Apply stat modifiers for an abnormal effect
---@param entity any The entity to apply modifiers to
---@param skill_definition table The skill definition containing stats table
---@param level number The skill level
function AbnormalEffects.apply_stat_modifiers(entity, skill_definition, level)
    if not skill_definition.tables or not skill_definition.tables.stats then
        return
    end

    local abnormal_kind = skill_definition.other.abnormalKind
    local priority = skill_definition.other.priority or 1

    for stat, operation_data in pairs(skill_definition.tables.stats) do
        local source_key = "abnormal:" .. abnormal_kind .. ":" .. stat
        StatModifiers.apply_modifier_with_source(entity, stat, operation_data, level, source_key, priority)
    end
end

-- Remove stat modifiers for an abnormal effect by kind
---@param entity any The entity to remove modifiers from
---@param abnormal_kind string The abnormal kind to remove
function AbnormalEffects.remove_stat_modifiers(entity, abnormal_kind)
    StatModifiers.remove_modifiers_containing(entity, "abnormal:" .. abnormal_kind)
end

-- Create effects over time from overTimeEffects definition
---@param entity any The entity to get level from for calculations
---@param skill_ref any The skill reference containing level
---@param over_time_effects_def table The overTimeEffects table from skill definition
---@return table Array of EffectOverTime objects
function AbnormalEffects.create_effects_over_time(entity, skill_ref, over_time_effects_def)
    local effects_over_time = {}

    if not over_time_effects_def then
        return effects_over_time
    end

    local char_level = Stats.get(entity, "ProgressLevelStats", "Level")

    for stat_type, operation_data in pairs(over_time_effects_def) do
        local operation_type = operation_data[1] -- Add, Sub, etc.
        local values = operation_data[2]         -- Array of values or single value
        local power = operation_data[3]          -- Stat usage power
        local interval = operation_data[4]       -- Individual interval for this operation

        if not interval or interval <= 0 then
            error("Missing or invalid interval for " .. stat_type .. " operation in overTimeEffects")
        end

        -- Get the value for current skill level or use single value
        local base_value
        if type(values) == "table" and #values > 0 then
            base_value = values[skill_ref.level._1] or values[1]
        else
            base_value = values
        end

        -- Use base_value if provided, otherwise calculate using leveled formula
        local effect_amount
        if base_value ~= nil and base_value ~= 0 then
            effect_amount = base_value
        else
            effect_amount = power * interval * ((char_level - 1) / 7.5)
        end

        local timer_mode_variant = construct(types.TimerMode, { variant = "Repeating" })
        local duration = Duration.from_millis(interval * 1000)
        local timer = Timer.new(duration, timer_mode_variant)
        local stat_kind = Stats.kind(stat_type)

        local stats_operation = construct(types["StatsOperation<f32>"],
            { variant = operation_type, _1 = effect_amount })

        local effect_over_time = construct(types.EffectOverTime, {
            timer = timer,
            stat = stat_kind,
            operation = stats_operation
        })

        table.insert(effects_over_time, effect_over_time)
    end

    return effects_over_time
end

-- Launch a toggle skill (activate or deactivate)
---@param entity any The entity using the skill
---@param skill_ref any The skill reference
---@param skill_definition table The skill definition
function AbnormalEffects.launch_toggle_skill(entity, skill_ref, skill_definition)
    local has_effect = AbnormalEffect.has_effect({ entity, skill_ref.id })

    if has_effect then
        AbnormalEffects.remove_effect(entity, skill_ref.id, skill_definition)
        return
    end

    -- Create effects over time from overTimeEffects definition
    local effects_over_time = AbnormalEffects.create_effects_over_time(entity, skill_ref,
        skill_definition.other.overTimeEffects)

    -- Create and apply the abnormal effect
    if #effects_over_time > 0 or skill_definition.other.abnormalKind then
        local effect, timer_data = AbnormalEffects.infinite_effect(
            skill_ref,
            skill_definition.other.abnormalKind,
            effects_over_time
        )
        AbnormalEffects.apply_effect(entity, effect, timer_data, skill_definition)
    end
end

-- Launches a restore skill that restores HP/MP/CP to the target
---@param entity any The entity casting the skill
---@param target_entity any The entity receiving the restoration
---@param skill_ref any The skill reference
---@param skill_definition table The skill definition
---@param restore_type string Optional restore type: "Hp", "Mp", or "Cp" (defaults to "Hp")
function AbnormalEffects.launch_restore_skill(entity, target_entity, skill_ref, skill_definition, restore_type)
    restore_type = restore_type or "Hp"

    local mp_consume = skill_definition.tables.mpConsume[skill_ref.level._1]
    Stats.consume_mp(entity, mp_consume)

    local restore_amount = skill_definition.tables.power[skill_ref.level._1]
    Stats.apply_restore(target_entity, restore_amount, restore_type)

    -- System message IDs for different restore types
    local restore_messages = {
        Hp = 1067, -- $s1 has been healed by $s2 HP.
        Mp = 1069, -- $s1 has recovered $s2 MP.
        Cp = 1406  -- $s1 has been restored by $s2 CP.
    }

    local entity_name = world.get_component(entity, types.Name).name
    local params = {
        SystemMessage.create_param("Text", entity_name),
        SystemMessage.create_param("Number", restore_amount)
    }
    SystemMessage.send(target_entity, restore_messages[restore_type], params)
end

-- Process abnormal effects changes for a single entity
---@param entity any The entity to process
---@param skills_storage table The skills storage to look up skill definitions
function AbnormalEffects.process_abnormal_changes(entity, skills_storage)
    local abnormal_effects = world.get_component(entity, types.AbnormalEffects)
    local stat_modifiers = world.get_component(entity, types.StatModifiers)

    if not abnormal_effects or not stat_modifiers then
        return
    end

    local diff_result = AbnormalEffect.diff(entity)

    local added_effects_ref = diff_result[1]
    local removed_effects_ref = diff_result[2]
    if not added_effects_ref or not removed_effects_ref then
        Logger.error("Failed to get diff results for entity " .. tostring(entity))
        return
    end

    -- Remove stat modifiers for effects that were removed
    local removed_count = #removed_effects_ref
    if removed_count > 0 then
        for i = 1, removed_count do
            local removed_effect = removed_effects_ref[i]
            if removed_effect then
                local skill_def = skills_storage.get(removed_effect.skill.id._1)
                if skill_def and skill_def.definition and skill_def.definition.other and skill_def.definition.other.abnormalKind then
                    AbnormalEffects.remove_stat_modifiers(entity, skill_def.definition.other.abnormalKind)
                end
            end
        end
    end

    -- Apply modifiers for newly added effects
    local added_count = #added_effects_ref
    if added_count > 0 then
        for i = 1, added_count do
            local added_effect = added_effects_ref[i]
            if added_effect then
                local skill_def = skills_storage.get(added_effect.skill.id._1)
                if skill_def and skill_def.apply_abnormal then
                    skill_def.apply_abnormal(entity, added_effect.skill)
                end
            end
        end
    end

    send_abnormal_status_update(entity)
end

return AbnormalEffects
