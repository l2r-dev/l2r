use crate::{
    clan::{ClanRank, castle::CastleId},
    stats::*,
};
use l2r_core::model::race::Race;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
    Player(PlayerCondition),
    Target(TargetCondition),
}

impl Condition {
    pub fn check_player(&self, player: &PlayerCondition) -> bool {
        match self {
            Condition::And(conditions) => {
                // All conditions must be true
                conditions.iter().all(|c| c.check_player(player))
            }
            Condition::Or(conditions) => {
                // At least one condition must be true
                conditions.iter().any(|c| c.check_player(player))
            }
            Condition::Not(condition) => !condition.check_player(player),
            Condition::Player(player_condition) => {
                let mut result = true;

                result &= player.fly_mounted == player_condition.fly_mounted;

                if !player_condition.races.is_empty() {
                    result &= player_condition
                        .races
                        .iter()
                        .any(|race| player.races.contains(race));
                }

                result &= player.cloak_status == player_condition.cloak_status;

                if !player_condition.instance_id.is_empty() {
                    result &= player_condition
                        .instance_id
                        .iter()
                        .all(|id| player.instance_id.contains(id));
                }

                if player_condition.level > 0.into() {
                    result &= player.level >= player_condition.level;
                }

                let (min_level, max_level) = player_condition.level_range;
                if min_level > 0.into() || max_level > 0.into() {
                    result &= player.level >= min_level && player.level <= max_level;
                }

                result &= player.siege_zone == player_condition.siege_zone;

                result &= player.pvp_flagged == player_condition.pvp_flagged;

                result &= player.hero == player_condition.hero;

                result &= player.clan_rank >= player_condition.clan_rank;

                match player_condition.castle {
                    CastleCondition::Any(_) => {
                        result &= player.castle != CastleCondition::None;
                    }
                    CastleCondition::Some(castle_id) => {
                        result &= player.castle == CastleCondition::Some(castle_id);
                    }
                    CastleCondition::None => {
                        result &= player.castle == CastleCondition::None;
                    }
                }

                result &= player.gender == player_condition.gender;

                result
            }

            _ => true,
        }
    }

    pub fn check_target(&self, target: &TargetCondition) -> bool {
        match self {
            Condition::And(conditions) => {
                // All conditions must be true
                conditions.iter().all(|c| c.check_target(target))
            }
            Condition::Or(conditions) => {
                // At least one condition must be true
                conditions.iter().any(|c| c.check_target(target))
            }
            Condition::Not(condition) => !condition.check_target(target),
            Condition::Target(target_condition) => {
                // Check if the target's level is within the specified range
                let (min_level, max_level) = target_condition.level_range;
                target.level_range.0 >= min_level && target.level_range.1 <= max_level
            }
            _ => true,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PlayerCondition {
    fly_mounted: bool,
    races: Vec<Race>,
    cloak_status: bool,
    instance_id: Vec<u32>,
    level: Level,
    level_range: (Level, Level),
    siege_zone: bool,
    pvp_flagged: bool,
    hero: bool,
    clan_rank: ClanRank,
    castle: CastleCondition,
    gender: Gender,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum AnyCastle {
    #[default]
    Any,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CastleCondition {
    Any(AnyCastle),
    Some(CastleId),
    #[default]
    None,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TargetCondition {
    level_range: (Level, Level),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_condition_true() {
        let condition = Condition::Player(PlayerCondition {
            level_range: (Level::from(20u32), Level::from(30u32)),
            fly_mounted: true,
            ..Default::default()
        });

        let player = PlayerCondition {
            fly_mounted: true,
            level: Level::from(25u32),
            ..Default::default()
        };

        assert!(condition.check_player(&player));
    }

    #[test]
    fn test_player_condition_false() {
        let condition = Condition::Player(PlayerCondition {
            level: Level::from(25u32),
            ..Default::default()
        });

        let player = PlayerCondition {
            level: Level::from(15u32),
            ..Default::default()
        };

        assert!(!condition.check_player(&player));
    }

    #[test]
    fn test_player_gender() {
        let condition = Condition::Player(PlayerCondition {
            gender: Gender::Male,
            ..Default::default()
        });

        let player = PlayerCondition {
            gender: Gender::Male,
            ..Default::default()
        };

        assert!(condition.check_player(&player));
    }

    #[test]
    fn test_player_condition_castle() {
        let condition = Condition::Player(PlayerCondition {
            castle: CastleCondition::Some(CastleId::Aden),
            ..Default::default()
        });

        let player = PlayerCondition {
            castle: CastleCondition::Some(CastleId::Aden),
            ..Default::default()
        };

        assert!(condition.check_player(&player));
    }

    // check castle condition serialize from json string
    #[test]
    fn test_player_condition_castle_serialize() {
        let castle_condition = r#"
        {
            "castle": "Aden"
        }
        "#;
        let condition: PlayerCondition = serde_json::from_str(castle_condition).unwrap();
        assert_eq!(condition.castle, CastleCondition::Some(CastleId::Aden));
    }

    #[test]
    fn test_and_condition() {
        let and_condition = Condition::And(vec![
            Condition::Player(PlayerCondition {
                races: vec![Race::Human, Race::Elf],
                ..Default::default()
            }),
            Condition::Player(PlayerCondition {
                level: 10.into(),
                ..Default::default()
            }),
        ]);

        let player = PlayerCondition {
            races: vec![Race::Elf],
            level: 25.into(),
            ..Default::default()
        };

        assert!(and_condition.check_player(&player));
    }

    #[test]
    fn test_or_condition() {
        let or_condition = Condition::Or(vec![
            Condition::Player(PlayerCondition {
                level: 5.into(),
                ..Default::default()
            }),
            Condition::Player(PlayerCondition {
                fly_mounted: true,
                ..Default::default()
            }),
        ]);

        let player = PlayerCondition {
            level: 10.into(),
            ..Default::default()
        };

        assert!(or_condition.check_player(&player));

        let player = PlayerCondition {
            fly_mounted: true,
            ..Default::default()
        };

        assert!(or_condition.check_player(&player));
    }

    #[test]
    fn test_not_condition() {
        let not_condition = Condition::Not(Box::new(Condition::Player(PlayerCondition {
            level: 10.into(),
            ..Default::default()
        })));

        let player = PlayerCondition {
            level: 9.into(),
            ..Default::default()
        };

        assert!(not_condition.check_player(&player));
    }

    #[test]
    fn test_target_condition_true() {
        let condition = Condition::Target(TargetCondition {
            level_range: (10.into(), 20.into()),
        });

        let target = TargetCondition {
            level_range: (15.into(), 15.into()),
        };

        assert!(condition.check_target(&target));
    }
}
