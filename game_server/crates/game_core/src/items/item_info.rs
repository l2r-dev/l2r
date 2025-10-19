use super::{AmmoKind, ConsumableKind, ItemsDataTable, UsableItem, WeaponKind, kind::SortingKind};
use crate::{items, stats::*};
use bevy::{log, platform::collections::HashMap, prelude::*};
use bevy_defer::{AsyncAccess, AsyncWorld};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
pub struct ItemInfo {
    display_id: Option<super::Id>,
    #[serde(default)]
    name: String,
    kind: super::kind::Kind,
    #[serde(default)]
    grade: super::grade::Grade,
    #[serde(default)]
    quality: super::quality::Quality,
    #[serde(default)]
    description: String,
    #[serde(default)]
    icon: String,
    #[serde(default)]
    crystal_count: u32,
    #[serde(default)]
    material: super::kind::ItemMaterial,
    #[serde(default)]
    weight: u32,
    #[serde(default)]
    price: u32,
    #[serde(default)]
    enchantable: bool,
    #[serde(default)]
    elementable: bool,
    stats: Option<HashMap<StatKind, StatsOperation<f32>>>,
    #[reflect(ignore)]
    conditions: Option<super::condition::Condition>,
    #[serde(default = "default_true")]
    tradable: bool,
    #[serde(default = "default_true")]
    dropable: bool,
    #[serde(default = "default_true")]
    sellable: bool,
    #[serde(default = "default_true")]
    depositable: bool,
    #[serde(default)]
    stackable: bool,
    #[serde(default)]
    premium: bool,
    item_skills: Option<Vec<ItemSkill>>,
    unequip_skills: Option<Vec<ItemSkill>>,
    mana: Option<u32>,
    time: Option<u32>,
    #[serde(default)]
    olympiad_restricted: bool,
    #[serde(default)]
    pet_useable: bool,
}
impl ItemInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn display_id(&self) -> Option<super::Id> {
        self.display_id
    }

    pub fn kind(&self) -> &super::kind::Kind {
        &self.kind
    }

    pub fn sorting_kind(&self) -> SortingKind {
        SortingKind::from(&self.kind)
    }

    pub fn bodypart(&self) -> Option<super::BodyPart> {
        let bodypart = super::BodyPart::try_from(&self.kind);
        bodypart.ok()
    }

    pub fn grade(&self) -> super::grade::Grade {
        self.grade
    }

    pub fn quality(&self) -> super::quality::Quality {
        self.quality
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn icon(&self) -> &str {
        &self.icon
    }

    pub fn crystal_count(&self) -> u32 {
        self.crystal_count
    }

    pub fn material(&self) -> super::kind::ItemMaterial {
        self.material
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn price(&self) -> u32 {
        self.price
    }

    pub fn is_enchantable(&self) -> bool {
        self.enchantable
    }

    pub fn is_elementable(&self) -> bool {
        self.elementable
    }

    pub fn stats_modifiers(&self) -> Option<StatModifiers> {
        if let Some(stats) = &self.stats {
            let mut modifiers = StatModifiers::default();
            let name = self.name.replace(" ", "_").to_lowercase();

            for (stat, operation) in stats.iter() {
                let source = format!("item:{}:{}", name, stat.to_string().to_lowercase());
                modifiers.add_modifier(
                    source,
                    StatModifier {
                        stat: *stat,
                        operation: *operation,
                        priority: 0,
                    },
                );
            }

            if let super::kind::Kind::Weapon(ref weapon) = self.kind {
                modifiers.add_modifier(
                    format!("item:{name}:range"),
                    StatModifier {
                        stat: StatKind::Attack(AttackStat::PAtkRange),
                        operation: StatsOperation::Set(*weapon.range as f32),
                        priority: 0,
                    },
                );

                modifiers.add_modifier(
                    format!("item:{name}:random_damage"),
                    StatModifier {
                        stat: StatKind::Attack(AttackStat::PAtkRandom),
                        operation: StatsOperation::Set(weapon.random_damage as f32),
                        priority: 0,
                    },
                );

                modifiers.add_modifier(
                    format!("item:{name}:width"),
                    StatModifier {
                        stat: StatKind::Attack(AttackStat::PAtkWidth),
                        operation: StatsOperation::Set(*weapon.width as f32),
                        priority: 0,
                    },
                );

                if let WeaponKind::Pole = weapon.kind {
                    modifiers.add_modifier(
                        format!("item:{name}:p_atk_max_targets_count"),
                        StatModifier {
                            stat: StatKind::Attack(AttackStat::PAtkMaxTargetsCount),
                            //TODO: вынести в конфиг
                            operation: StatsOperation::Set(3.0),
                            priority: 0,
                        },
                    )
                }
            }

            Some(modifiers)
        } else {
            None
        }
    }

    pub fn conditions(&self) -> Option<&super::condition::Condition> {
        self.conditions.as_ref()
    }

    pub fn tradable(&self) -> bool {
        self.tradable
    }

    pub fn dropable(&self) -> bool {
        self.dropable
    }

    pub fn sellable(&self) -> bool {
        self.sellable
    }

    pub fn depositable(&self) -> bool {
        self.depositable
    }

    pub fn stackable(&self) -> bool {
        self.stackable
    }

    pub async fn is_stackable_async(item_id: super::Id) -> bool {
        AsyncWorld
            .resource::<ItemsDataTable>()
            .get(|table| {
                AsyncWorld
                    .resource::<Assets<super::ItemsInfo>>()
                    .get(|assets| {
                        table
                            .get_item_info(item_id, assets)
                            .map(|item_info| item_info.stackable())
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn ammo_matches(&self, ammo: &Self) -> bool {
        if self.grade.arrow_grade() != ammo.grade {
            return false;
        }

        let items::Kind::Weapon(v) = self.kind else {
            return false;
        };

        match v.kind {
            WeaponKind::Bow => {
                matches!(
                    ammo.kind,
                    items::Kind::Consumable(ConsumableKind::Ammo(AmmoKind::Arrow))
                )
            }

            WeaponKind::Crossbow => {
                matches!(
                    ammo.kind,
                    items::Kind::Consumable(ConsumableKind::Ammo(AmmoKind::Bolt))
                )
            }

            _ => false,
        }
    }

    pub fn premium(&self) -> bool {
        self.premium
    }

    pub fn item_skills(&self) -> Option<&Vec<ItemSkill>> {
        self.item_skills.as_ref()
    }

    pub fn unequip_skills(&self) -> Option<&Vec<ItemSkill>> {
        self.unequip_skills.as_ref()
    }

    pub fn mana(&self) -> Option<u32> {
        self.mana
    }

    pub fn time(&self) -> Option<u32> {
        self.time
    }

    pub fn olympiad_restricted(&self) -> bool {
        self.olympiad_restricted
    }

    pub fn pet_useable(&self) -> bool {
        self.pet_useable
    }
    pub fn use_item(&self) {
        if self.usable()
            && let Some(item_skills) = self.item_skills()
        {
            for item_skill in item_skills {
                log::info!("Use item skill: {:?}", item_skill);
            }
        }
    }
}

impl super::UsableItem for ItemInfo {
    fn usable(&self) -> bool {
        self.kind.usable()
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Reflect, Serialize)]
pub struct ItemSkill {
    pub id: u32,
    pub level: u32,
}
