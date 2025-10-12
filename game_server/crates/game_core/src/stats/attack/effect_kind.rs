use crate::items::WeaponKind;
use bevy::{platform::collections::HashMap, prelude::*};
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

pub struct EffectKindComponentsPlugin;
impl Plugin for EffectKindComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AttackEffects>()
            .register_type::<DefenceEffects>()
            .register_type::<EffectKind>()
            .register_type::<Weapon>()
            .register_type::<Weakness>()
            .register_type::<Resistance>();
    }
}

#[derive(Clone, Component, Debug, Reflect)]
pub struct AttackEffects {
    weapon: Weapon,
    weakness: HashMap<Weakness, f32>,
    resistance: HashMap<Resistance, f32>,
}

impl Default for AttackEffects {
    fn default() -> Self {
        Self {
            weapon: Weapon::default(),
            weakness: HashMap::with_capacity(Weakness::iter().count() / 2),
            resistance: HashMap::with_capacity(Resistance::iter().count() / 2),
        }
    }
}

#[derive(Clone, Component, Debug, Reflect)]
pub struct DefenceEffects {
    effects: HashMap<EffectKind, f32>,
    vulnerabilities: Vec<EffectKind>,
}
impl Default for DefenceEffects {
    fn default() -> Self {
        let mut effects = HashMap::with_capacity(EffectKind::count());

        for weapon in Weapon::iter() {
            effects.insert(EffectKind::Weapon(weapon), 1.0);
        }

        Self {
            effects,
            vulnerabilities: Vec::with_capacity(EffectKind::count()),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Reflect,
    IntoPrimitive,
    FromPrimitive,
    EnumIter,
    Hash,
)]
#[repr(u32)]
pub enum Weapon {
    Sword,
    Blunt,
    Dagger,
    Pole,
    #[default]
    Fist,
    Bow,
    Etc,
    Dual,
    DualFist,
    Rapier,
    Crossbow,
    AncientSword,
    DualDagger,
}

impl From<WeaponKind> for Weapon {
    fn from(weapon_kind: WeaponKind) -> Self {
        match weapon_kind {
            WeaponKind::Sword(_) => Weapon::Sword,
            WeaponKind::Blunt(_) => Weapon::Blunt,
            WeaponKind::Bow => Weapon::Bow,
            WeaponKind::Crossbow => Weapon::Crossbow,
            WeaponKind::Dagger(_) => Weapon::Dagger,
            WeaponKind::Fist(_) => Weapon::Fist,
            WeaponKind::Etc => Weapon::Etc,
            WeaponKind::FortFlag => Weapon::Etc,
            WeaponKind::FishingRod => Weapon::Etc,
            WeaponKind::Pole => Weapon::Pole,
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Reflect,
    IntoPrimitive,
    FromPrimitive,
    EnumIter,
    Hash,
)]
#[repr(u32)]
pub enum Weakness {
    #[default]
    Bug,
    Animal,
    Plant,
    Beast,
    Dragon,
    Giant,
    Construct,
    Valakas,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Reflect,
    IntoPrimitive,
    FromPrimitive,
    EnumIter,
    Hash,
)]
#[repr(u32)]
pub enum Resistance {
    #[default]
    Poison,
    Hold,
    Bleed,
    Sleep,
    Shock,
    Derangement,
    Paralyze,
    Boss,
    Death,
    Anesthesia,
    CriticalPoison,
    RootPhysically,
    RootMagically,
    TurnStone,
    Gust,
    PhysicalBlockade,
    Target,
    Physical,
    Magical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum EffectKind {
    Weapon(Weapon),
    Weakness(Weakness),
    Resistance(Resistance),
}

impl Default for EffectKind {
    fn default() -> Self {
        EffectKind::Weapon(Weapon::default())
    }
}

impl EffectKind {
    pub fn kind(&self) -> u32 {
        match self {
            EffectKind::Weapon(_) => 1,
            EffectKind::Weakness(_) => 2,
            EffectKind::Resistance(_) => 3,
        }
    }

    pub fn count() -> usize {
        Weapon::iter().count() + Weakness::iter().count() + Resistance::iter().count()
    }
}

impl From<EffectKind> for u32 {
    fn from(damage_type: EffectKind) -> u32 {
        match damage_type {
            EffectKind::Weapon(weapon) => weapon.into(),
            EffectKind::Weakness(weakness) => 1000 + u32::from(weakness),
            EffectKind::Resistance(resistance) => 2000 + u32::from(resistance),
        }
    }
}

impl From<u32> for EffectKind {
    fn from(value: u32) -> Self {
        match value {
            0..=999 => {
                let wd = Weapon::from_primitive(value);
                EffectKind::Weapon(wd)
            }
            1000..=1999 => {
                let weak_value = value - 1000;
                let weak = Weakness::from_primitive(weak_value);
                EffectKind::Weakness(weak)
            }
            2000..=2999 => {
                let res_value = value - 2000;
                let res = Resistance::from_primitive(res_value);
                EffectKind::Resistance(res)
            }
            _ => EffectKind::default(),
        }
    }
}

impl From<f32> for EffectKind {
    fn from(value: f32) -> Self {
        EffectKind::from(value as u32)
    }
}

impl From<EffectKind> for f32 {
    fn from(value: EffectKind) -> Self {
        u32::from(value) as f32
    }
}

impl Serialize for EffectKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = match self {
            EffectKind::Weapon(stat) => format!("{stat:?}"),
            EffectKind::Weakness(stat) => format!("{stat:?}"),
            EffectKind::Resistance(stat) => format!("{stat:?}"),
        };

        serializer.serialize_str(&str_value)
    }
}

impl<'de> Deserialize<'de> for EffectKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EffectKindVisitor;

        impl<'de> Visitor<'de> for EffectKindVisitor {
            type Value = EffectKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string representing an effect kind (Weapon, Weakness, or Resistance)",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<EffectKind, E>
            where
                E: de::Error,
            {
                for stat in Weapon::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(EffectKind::Weapon(stat));
                    }
                }

                for stat in Weakness::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(EffectKind::Weakness(stat));
                    }
                }

                for stat in Resistance::iter() {
                    if format!("{stat:?}") == value {
                        return Ok(EffectKind::Resistance(stat));
                    }
                }

                Err(E::custom(format!("Unknown effect kind: {value}")))
            }
        }

        deserializer.deserialize_str(EffectKindVisitor)
    }
}

impl AttackEffects {
    pub fn get(&self, effect_kind: EffectKind) -> f32 {
        match effect_kind {
            EffectKind::Weapon(_) => 2.0,
            EffectKind::Weakness(weakness) => self.get_weakness(weakness),
            EffectKind::Resistance(resistance) => self.get_resistance(resistance),
        }
    }

    pub fn has(&self, effect_kind: EffectKind) -> bool {
        self.get(effect_kind) > 0.0
    }

    pub fn get_weapon(&self) -> Weapon {
        self.weapon
    }

    pub fn set_weapon(&mut self, effect: Weapon) {
        self.weapon = effect;
    }

    pub fn get_weakness(&self, weakness: Weakness) -> f32 {
        self.weakness.get(&weakness).copied().unwrap_or(0.0)
    }

    pub fn set_weakness(&mut self, weakness: Weakness, value: f32) {
        self.weakness.insert(weakness, value);
    }

    pub fn get_resistance(&self, resistance: Resistance) -> f32 {
        self.resistance.get(&resistance).copied().unwrap_or(0.0)
    }

    pub fn set_resistance(&mut self, resistance: Resistance, value: f32) {
        self.resistance.insert(resistance, value);
    }
}

impl DefenceEffects {
    pub fn get(&self, effect_kind: EffectKind) -> f32 {
        self.effects.get(&effect_kind).copied().unwrap_or(0.0)
    }

    pub fn has(&self, effect_kind: EffectKind) -> bool {
        self.get(effect_kind) > 0.0
    }

    pub fn set(&mut self, effect_kind: EffectKind, value: f32) {
        self.effects.insert(effect_kind, value);
    }

    pub fn add(&mut self, effect_kind: EffectKind, value: f32) {
        let current_value = self.get(effect_kind);
        self.effects.insert(effect_kind, current_value + value);
    }

    pub fn is_invulnerable(&self, effect_kind: EffectKind) -> bool {
        self.vulnerabilities.contains(&effect_kind)
    }

    pub fn set_invulnerability(&mut self, effect_kind: EffectKind) {
        self.vulnerabilities.push(effect_kind);
    }

    pub fn remove_invulnerability(&mut self, effect_kind: EffectKind) {
        self.vulnerabilities.retain(|&x| x != effect_kind);
    }
}
