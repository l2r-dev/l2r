use super::GameServerPacketCodes;
use crate::stats::*;
use bevy::prelude::*;
use l2r_core::{
    model::{base_class::BaseClass, race::Race},
    packets::{L2rServerPacket, ServerPacketBuffer},
};
use std::{collections::HashMap, fmt};

#[derive(Clone, Default, Reflect)]
pub struct ResponseCharCreateMenu {
    new_character_races: HashMap<(Race, BaseClass), PrimalStats>,
}

impl fmt::Debug for ResponseCharCreateMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl L2rServerPacket for ResponseCharCreateMenu {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::NEW_CHARACTER_SUCCESS.to_le_bytes());
        buffer.u32_from_usize(self.new_character_races.len());
        for ((race, base_class), primal_stats) in self.new_character_races {
            buffer.u32(race.into());
            buffer.u32(base_class.into());
            buffer.u32(0x46);
            buffer.u32(primal_stats.get(PrimalStat::STR));
            buffer.u32(0x0A);
            buffer.u32(0x46);
            buffer.u32(primal_stats.get(PrimalStat::DEX));
            buffer.u32(0x0A);
            buffer.u32(0x46);
            buffer.u32(primal_stats.get(PrimalStat::CON));
            buffer.u32(0x0A);
            buffer.u32(0x46);
            buffer.u32(primal_stats.get(PrimalStat::INT));
            buffer.u32(0x0A);
            buffer.u32(0x46);
            buffer.u32(primal_stats.get(PrimalStat::WIT));
            buffer.u32(0x0A);
            buffer.u32(0x46);
            buffer.u32(primal_stats.get(PrimalStat::MEN));
            buffer.u32(0x0A);
        }
        buffer
    }
}

impl ResponseCharCreateMenu {
    pub fn new(race_stats: &RaceStats) -> Self {
        let default_races = Race::default_races();
        let default_base_classes = BaseClass::default_classes();
        let kamael_base_classes = BaseClass::kamael_classes();

        let mut new_character_races = HashMap::new();

        for race in &default_races {
            for base_class in &default_base_classes {
                let base_class_stats = race_stats.get(*race, *base_class);
                new_character_races
                    .insert((*race, *base_class), base_class_stats.primal_stats.clone());
            }
        }
        let kamael_race = Race::Kamael;
        for base_class in &kamael_base_classes {
            let kamael_base_class_stats_info = race_stats.get(kamael_race, *base_class);
            new_character_races.insert(
                (kamael_race, *base_class),
                kamael_base_class_stats_info.primal_stats.clone(),
            );
        }

        Self {
            new_character_races,
        }
    }
}
