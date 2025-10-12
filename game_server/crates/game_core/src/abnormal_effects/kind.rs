use bevy::prelude::*;
use derive_more::From;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};
use std::fmt;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};

#[derive(Clone, Copy, Debug, EnumDiscriminants, Eq, From, Hash, PartialEq, Reflect)]
#[strum_discriminants(name(AbnormalKindCategory))]
#[strum_discriminants(derive(Display, EnumString, EnumIter, Hash, Reflect))]
pub enum AbnormalKind {
    Rhythm(RhythmKind),
    Buff(BuffKind),
    Debuff(DebuffKind),
}

impl fmt::Display for AbnormalKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AbnormalKind::Rhythm(kind) => write!(f, "{}", kind),
            AbnormalKind::Buff(kind) => write!(f, "{}", kind),
            AbnormalKind::Debuff(kind) => write!(f, "{}", kind),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Display, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize,
)]
pub enum RhythmKind {
    // Dance effects
    DanceDefenceMotion1,
    DanceOfAlignment,
    DanceOfAquaGuard,
    DanceOfBerserker,
    DanceOfBladestorm,
    DanceOfConcentration,
    DanceOfEarthGuard,
    DanceOfFire,
    DanceOfFury,
    DanceOfInspiration,
    DanceOfLight,
    DanceOfMystic,
    DanceOfProtection,
    DanceOfShadow,
    DanceOfSiren,
    DanceOfVampire,
    DanceOfWarrior,
    // Song effects
    SongBattleWhisper,
    SongOfChampion,
    SongOfEarth,
    SongOfElemental,
    SongOfFlameGuard,
    SongOfHunter,
    SongOfInvocation,
    SongOfLife,
    SongOfMeditation,
    SongOfPurification,
    SongOfRenewal,
    SongOfStormGuard,
    SongOfVengeance,
    SongOfVitality,
    SongOfWarding,
    SongOfWater,
    SongOfWind,
    SongOfWindstorm,
}

#[derive(
    Clone, Copy, Debug, Display, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize,
)]
pub enum BuffKind {
    AbilityChange,
    AllRegenUp,
    ArmorEarth,
    ArmorFire,
    ArmorHoly,
    ArmorUnholy,
    ArmorWater,
    ArmorWind,
    AttackSpeedUpBow,
    AttackTimeUp,
    AttributePotion,
    AvoidUp,
    AvoidUpSpecial,
    Berserker,
    BlessTheBlood,
    BlockShieldUp,
    BlockSpeedUp,
    BuffQueenOfCat,
    BuffUnicornSeraphim,
    CastingTimeDown,
    CheapMagic,
    Combination,
    CounterCritical,
    CounterSkill,
    CpUp,
    CriticalRateUp,
    CriticalDmgUp,
    CriticalDmgUpToggle,
    CriticalProbUp,
    DmgShield,
    DragonBuff,
    DuelistSpirit,
    DwarfAttackBuff,
    DwarfDefenceBuff,
    ElementalArmor,
    EvasionBuff,
    HealEffectUp,
    HealPowerUp,
    HeroBuff,
    Hide,
    HitUp,
    HolyAttack,
    HpRecover,
    HpRegenUp,
    ImproveCrtRateDmgUp,
    ImproveHitDefenceCrtRateUp,
    ImproveHpMpUp,
    ImproveMaMdUp,
    ImprovePaPdUp,
    ImproveShieldRateDefenceUp,
    ImproveSpeedAvoidUp,
    ImproveVampiricHaste,
    Instinct,
    Invincibility,
    IronShield,
    IronShieldI,
    KnightAura,
    LifeForceKamael,
    LifeForceOrc,
    LifeForceOthers,
    MagicalStance,
    MagicCriticalUp,
    Majesty,
    MaximumAbility,
    MaxBreathUp,
    MaxHpUp,
    MaxHpCpUp,
    MaxMpUp,
    MaMdUp,
    MaUp,
    MaUpHerb,
    MaUpSpecial,
    MdUp,
    MdUpAttr,
    MightMortal,
    Mirage,
    MpCostDown,
    MpRegenUp,
    MultiBuff,
    MultiBuffA,
    Patience,
    PaPdUp,
    PaUp,
    PaUpHerb,
    PaUpSpecial,
    PdUp,
    PdUpBow,
    PdUpSpecial,
    PhysicalStance,
    PkProtect,
    PotionOfGenesis,
    PreserveAbnormal,
    Protection,
    PvpWeaponBuff,
    RageMight,
    RechargeUp,
    ReduceDropPenalty,
    ReflectAbnormal,
    ReflectMagicDd,
    ResistBleeding,
    ResistDebuffDispel,
    ResistDerangement,
    ResistHolyUnholy,
    ResistPoison,
    ResistShock,
    ResistSpiritless,
    ResurrectionSpecial,
    ReuseDelayDown,
    SeedOfCritical,
    SeedOfKnight,
    ShieldDefenceUp,
    ShieldProbUp,
    SkillIgnore,
    Snipe,
    SpeedUp,
    SpeedUpSpecial,
    SsqTownBlessing,
    Stealth,
    SubTriggerCrtRateUp,
    SubTriggerDefence,
    SubTriggerHaste,
    SubTriggerSpirit,
    SuperHasteToggle,
    Talisman,
    TCrtDmgUp,
    TCrtRateUp,
    ThrillFight,
    TouchOfLife,
    UltimateBuff,
    ValakasItem,
    VampiricAttack,
    VampiricAttackSpecial,
    Vote,
    VpKeep,
    VpUp,
    WeaponMastery,
    Will,
    AbnormalInvincibility,
    AbnormalItem,
    AbHawkEye,
    Apella,
    ArcherSpecial,
    ArcherSpecialI,
    ArrowRain,
    AvoidSkill,
    BigBody,
    BigHead,
    BloodContract,
    BrEventBuf1,
    BrEventBuf10,
    BrEventBuf2,
    BrEventBuf3,
    BrEventBuf5,
    BrEventBuf6,
    BrEventBuf7,
    CounterCriticalTrigger,
    DamageAmplify,
    DdResist,
    Deathworm,
    EntryForGame,
    EventGawi,
    EventSantaReward,
    EventTerritory,
    EventWin,
    FinalSecret,
    FlameHawk,
    FlyAway,
    FocusDagger,
    ForceMeditation,
    ForceOfDestruction,
    GhostPiercing,
    HotGround,
    KamaelSpecial,
    Limit,
    None,
    NormalAttackBlock,
    PolearmAttack,
    PublicSlot,
    RealTarget,
    SignalA,
    SignalB,
    SignalC,
    SignalD,
    SignalE,
    SoaBuff1,
    SoaBuff2,
    SoaBuff3,
    SummonCondition,
    TimeCheck,
    WpChangeEvent,
}

#[derive(
    Clone, Copy, Debug, Display, EnumIter, Eq, Hash, PartialEq, Reflect, Serialize, Deserialize,
)]
pub enum DebuffKind {
    AllAttackDown,
    AllSpeedDown,
    Anesthesia,
    AntarasDebuff,
    AttackTimeDown,
    AttackTimeDownSpecial,
    AvoidDown,
    BetrayalMark,
    Bleeding,
    BlockResurrection,
    BlockTransform,
    BotPenalty,
    CancelProbDown,
    CastingTimeUp,
    CriticalDmgDown,
    CriticalPoison,
    CriticalProbDown,
    CurseLifeFlow,
    DarkSeed,
    DeathClack,
    DeathMark,
    DeathPenalty,
    DebuffNightshade,
    DebuffShield,
    DecreaseWeightPenalty,
    Derangement,
    DetectWeakness,
    Disarm,
    DotAttr,
    DotMp,
    DragonBreath,
    Enervation,
    EvilBlood,
    ExposeWeakPoint,
    FatalPoison,
    FireDot,
    FishingMasteryDown,
    Freezing,
    HealEffectDown,
    HeroDebuff,
    HitDown,
    HpRegenDown,
    MaDown,
    MaxHpDown,
    MdDown,
    MentalImpoverish,
    Meteor,
    MirageTrap,
    MpCostUp,
    MultiDebuff,
    MultiDebuffA,
    MultiDebuffB,
    MultiDebuffC,
    MultiDebuffD,
    MultiDebuffE,
    MultiDebuffF,
    MultiDebuffFire,
    MultiDebuffG,
    MultiDebuffHoly,
    MultiDebuffSoul,
    MultiDebuffUnholy,
    MultiDebuffWater,
    MultiDebuffWind,
    Oblivion,
    Paralyze,
    PaDown,
    PdDown,
    Pinch,
    Poison,
    Possession,
    PossessionSpecial,
    PvpDmgDown,
    PvpWeaponDebuff,
    ReuseDelayUp,
    RootMagically,
    RootPhysically,
    SeizureA,
    SeizureB,
    SeizureC,
    SeizurePenalty,
    Silence,
    SilenceAll,
    SilencePhysical,
    Sleep,
    SpaDiseaseA,
    SpaDiseaseB,
    SpaDiseaseC,
    SpaDiseaseD,
    SpeedDown,
    Spite,
    SpoilBomb,
    SsqTownCurse,
    StarFall,
    StigmaA,
    StigmaOfSilen,
    Stun,
    TargetLock,
    ThinSkin,
    TouchOfDeath,
    TransferDamage,
    Transform,
    TransformHangover,
    TransformScrifice,
    TransformScrificeP,
    TurnFlee,
    TurnPassive,
    TurnStone,
    TCrtDmgDown,
    UltimateDebuff,
    Vibration,
    WatcherGaze,
    WaterDot,
    WeakConstitution,
    WindDot,
}

impl Serialize for AbnormalKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = match self {
            AbnormalKind::Rhythm(effect) => format!("{effect:?}"),
            AbnormalKind::Buff(effect) => format!("{effect:?}"),
            AbnormalKind::Debuff(effect) => format!("{effect:?}"),
        };
        serializer.serialize_str(&str_value)
    }
}

impl<'de> Deserialize<'de> for AbnormalKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AbnormalKindVisitor;

        impl<'de> Visitor<'de> for AbnormalKindVisitor {
            type Value = AbnormalKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing an abnormal effect kind")
            }

            fn visit_str<E>(self, value: &str) -> Result<AbnormalKind, E>
            where
                E: de::Error,
            {
                for effect in RhythmKind::iter() {
                    if format!("{effect:?}") == value {
                        return Ok(AbnormalKind::Rhythm(effect));
                    }
                }

                for effect in BuffKind::iter() {
                    if format!("{effect:?}") == value {
                        return Ok(AbnormalKind::Buff(effect));
                    }
                }

                for effect in DebuffKind::iter() {
                    if format!("{effect:?}") == value {
                        return Ok(AbnormalKind::Debuff(effect));
                    }
                }

                Err(E::custom(format!("Unknown abnormal effect kind: {value}")))
            }
        }

        deserializer.deserialize_str(AbnormalKindVisitor)
    }
}

impl AbnormalKind {
    pub fn is_same(&self, other: &AbnormalKind) -> bool {
        self == other
    }

    pub fn category(&self) -> AbnormalKindCategory {
        AbnormalKindCategory::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_same_effect_identical_effects() {
        let effect1 = AbnormalKind::Rhythm(RhythmKind::DanceOfFire);
        let effect2 = AbnormalKind::Rhythm(RhythmKind::DanceOfFire);

        assert!(effect1.is_same(&effect2));
        assert!(effect2.is_same(&effect1));
    }

    #[test]
    fn test_is_same_effect_different_effects() {
        let effect1 = AbnormalKind::Rhythm(RhythmKind::DanceOfFire);
        let effect2 = AbnormalKind::Rhythm(RhythmKind::DanceOfFury);
        let effect3 = AbnormalKind::Buff(BuffKind::HpRegenUp);

        assert!(!effect1.is_same(&effect2));
        assert!(!effect2.is_same(&effect1));
        assert!(!effect2.is_same(&effect3));
        assert!(!effect3.is_same(&effect1));
    }
}
