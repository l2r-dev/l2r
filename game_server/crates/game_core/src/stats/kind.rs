use crate::stats::*;
use derive_more::From;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};
use std::fmt;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString};

#[derive(Clone, Copy, Debug, EnumDiscriminants, Eq, From, Hash, PartialEq, Reflect)]
#[strum_discriminants(name(StatKindVariants))]
#[strum_discriminants(derive(Display, EnumString, EnumIter, Hash))]
pub enum StatKind {
    Vitals(VitalsStat),
    Attack(AttackStat),
    Defence(DefenceStat),
    Movement(MovementStat),
    Critical(CriticalStat),
    Primal(PrimalStat),
    ElementPower(Element),
    Inventory(InventoryStat),
    MpConsumption(MpConsumptionStat),
    Progress(ProgressStat),
    ProgressLevel(ProgressLevelStat),
    ProgressRates(ProgressRatesStat),
    Other(OtherStat),
}

impl fmt::Display for StatKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StatKind::Vitals(stat) => format!("{stat:?}"),
            StatKind::Attack(stat) => format!("{stat:?}"),
            StatKind::Defence(stat) => format!("{stat:?}"),
            StatKind::Movement(stat) => format!("{stat:?}"),
            StatKind::Critical(stat) => format!("{stat:?}"),
            StatKind::Primal(stat) => format!("{stat:?}"),
            StatKind::ElementPower(stat) => format!("{stat:?}"),
            StatKind::Inventory(stat) => format!("{stat:?}"),
            StatKind::MpConsumption(stat) => format!("{stat:?}"),
            StatKind::Progress(stat) => format!("{stat:?}"),
            StatKind::ProgressLevel(stat) => format!("{stat:?}"),
            StatKind::ProgressRates(stat) => format!("{stat:?}"),
            StatKind::Other(stat) => format!("{stat:?}"),
        };
        write!(f, "{s}")
    }
}

impl Serialize for StatKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = match self {
            StatKind::Vitals(stat) => format!("{stat:?}"),
            StatKind::Attack(stat) => format!("{stat:?}"),
            StatKind::Defence(stat) => format!("{stat:?}"),
            StatKind::Movement(stat) => format!("{stat:?}"),
            StatKind::Critical(stat) => format!("{stat:?}"),
            StatKind::Primal(stat) => format!("{stat:?}"),
            StatKind::ElementPower(stat) => format!("{stat:?}"),
            StatKind::Inventory(stat) => format!("{stat:?}"),
            StatKind::MpConsumption(stat) => format!("{stat:?}"),
            StatKind::Progress(stat) => format!("{stat:?}"),
            StatKind::ProgressLevel(stat) => format!("{stat:?}"),
            StatKind::ProgressRates(stat) => format!("{stat:?}"),
            StatKind::Other(stat) => format!("{stat:?}"),
        };
        serializer.serialize_str(&str_value)
    }
}

impl<'de> Deserialize<'de> for StatKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatKindVisitor;

        impl<'de> Visitor<'de> for StatKindVisitor {
            type Value = StatKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a stat kind")
            }

            fn visit_str<E>(self, value: &str) -> Result<StatKind, E>
            where
                E: de::Error,
            {
                for stat in VitalsStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Vitals(stat));
                    }
                }

                for stat in AttackStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Attack(stat));
                    }
                }

                for stat in DefenceStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Defence(stat));
                    }
                }

                for stat in MovementStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Movement(stat));
                    }
                }

                for stat in CriticalStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Critical(stat));
                    }
                }

                for stat in PrimalStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Primal(stat));
                    }
                }

                for stat in Element::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::ElementPower(stat));
                    }
                }

                for stat in InventoryStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Inventory(stat));
                    }
                }

                for stat in MpConsumptionStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::MpConsumption(stat));
                    }
                }

                for stat in ProgressStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Progress(stat));
                    }
                }

                for stat in ProgressRatesStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::ProgressRates(stat));
                    }
                }

                for stat in OtherStat::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(StatKind::Other(stat));
                    }
                }

                Err(E::custom(format!("Unknown stat kind: {value}")))
            }
        }

        deserializer.deserialize_str(StatKindVisitor)
    }
}

impl From<StatKindVariants> for state::StatKindSystems {
    fn from(variants: StatKindVariants) -> Self {
        match variants {
            StatKindVariants::Vitals => Self::Vitals,
            StatKindVariants::Attack => Self::Attack,
            StatKindVariants::Defence => Self::Defence,
            StatKindVariants::Movement => Self::Movement,
            StatKindVariants::Critical => Self::Critical,
            StatKindVariants::Primal => Self::Primal,
            StatKindVariants::ElementPower => Self::ElementPower,
            StatKindVariants::Inventory => Self::Inventory,
            StatKindVariants::MpConsumption => Self::MpConsumption,
            StatKindVariants::Progress => Self::Progress,
            StatKindVariants::ProgressLevel => Self::ProgressLevel,
            StatKindVariants::ProgressRates => Self::ProgressRates,
            StatKindVariants::Other => Self::Other,
        }
    }
}
