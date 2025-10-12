use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use scripting::prelude::ScriptValue;
use sea_orm::{
    self, TryGetError, TryGetable,
    sea_query::{self, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(
    EnumIter,
    Display,
    Default,
    Copy,
    Component,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    IntoPrimitive,
    Reflect,
)]
#[repr(u32)]
pub enum ClassId {
    #[default]
    HumanFighter,
    Warrior,
    Gladiator,
    Warlord,
    HumanKnight,
    Paladin,
    DarkAvenger,
    Rogue,
    TreasureHunter,
    Hawkeye,
    HumanMystic,
    HumanWizard,
    Sorcerer,
    Necromancer,
    Warlock,
    Cleric,
    Bishop,
    Prophet,
    ElvenFighter,
    ElvenKnight,
    TempleKnight,
    SwordSinger,
    ElvenScout,
    PlainsWalker,
    SilverRanger,
    ElvenMystic,
    ElvenWizard,
    Spellsinger,
    ElementalSummoner,
    ElvenOracle,
    ElvenElder,
    DarkFighter,
    PalusKnight,
    ShillienKnight,
    Bladedancer,
    Assassin,
    AbyssWalker,
    PhantomRanger,
    DarkMystic,
    DarkWizard,
    Spellhowler,
    PhantomSummoner,
    ShillienOracle,
    ShillienElder,
    OrcFighter,
    OrcRaider,
    Destroyer,
    Monk,
    Tyrant,
    OrcMystic,
    OrcShaman,
    Overlord,
    Warcryer,
    DwarvenFighter,
    Scavenger,
    BountyHunter,
    Artisan,
    Warsmith = 57,
    Duelist = 88,
    Dreadnought,
    PhoenixKnight,
    HellKnight,
    Sagittarius,
    Adventurer,
    Archmage,
    Soultaker,
    ArcanaLord,
    Cardinal,
    Hierophant,
    EvasTemplar,
    SwordMuse,
    WindRider,
    MoonlightSentinel,
    MysticMuse,
    ElementalMaster,
    EvasSaint,
    ShillienTemplar,
    SpectralDancer,
    GhostHunter,
    GhostSentinel,
    StormScreamer,
    SpectralMaster,
    ShillienSaint,
    Titan,
    GrandKhavatari,
    Dominator,
    Doomcryer,
    FortuneSeeker,
    Maestro,
    SoldierMale = 123,
    SoldierFemale,
    Trooper,
    Warder,
    Berserker,
    SoulBreakerMale,
    SoulBreakerFemale,
    Arbalester,
    Doombringer,
    SoulHoundMale,
    SoulHoundFemale,
    Trickster,
    Inspector,
    Judicator,
}

impl TryFrom<i32> for ClassId {
    type Error = TryFromPrimitiveError<Self>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        ClassId::try_from_primitive(value as u32)
    }
}

impl From<ClassId> for Value {
    fn from(class_id: ClassId) -> Self {
        Value::Int(Some(class_id as i32))
    }
}

impl TryGetable for ClassId {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        idx: I,
    ) -> Result<Self, TryGetError> {
        let value: i32 = res.try_get_by(idx)?;
        <ClassId as std::convert::TryFrom<i32>>::try_from(value).map_err(|_| {
            TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "Failed to convert {value} to ClassId enum"
            )))
        })
    }
}

impl ValueType for ClassId {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::Int(Some(value)) = v {
            <ClassId as std::convert::TryFrom<i32>>::try_from(value).map_err(|_| ValueTypeErr)
        } else {
            Err(ValueTypeErr)
        }
    }

    fn type_name() -> String {
        stringify!(ClassId).to_owned()
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::Integer
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::Int
    }
}

impl TryFrom<&ScriptValue> for ClassId {
    type Error = ();

    fn try_from(value: &ScriptValue) -> Result<Self, Self::Error> {
        match value {
            ScriptValue::Integer(n) if *n == (*n).clamp(0, u32::MAX as i64) => {
                ClassId::try_from_primitive(*n as u32).map_err(|_| ())
            }
            ScriptValue::Float(n) if *n == n.clamp(0.0, u32::MAX as f64) && n.fract() == 0.0 => {
                ClassId::try_from_primitive(*n as u32).map_err(|_| ())
            }
            _ => Err(()),
        }
    }
}
