use crate::{
    character::{self},
    network::packets::client::CharSlot,
    stats::*,
};
use bevy::prelude::*;
use l2r_core::{
    model::race::Race,
    packets::{ClientPacketBuffer, L2rSerializeError},
};
use std::fmt::{
    Debug, {self},
};

#[derive(Clone, Event, PartialEq, Reflect)]
pub struct RequestCharCreate {
    pub name: String,
    pub race: Race,
    pub class_id: ClassId,
    pub primal_stats: PrimalStats,
    pub appearance: character::Appearance,
}

impl Debug for RequestCharCreate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}, {:?}",
            self.name, self.race, self.class_id, self.appearance
        )
    }
}

impl TryFrom<ClientPacketBuffer> for RequestCharCreate {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let name = buffer.str()?;

        let race = Race::try_from(buffer.i32()?)?;
        let gender = Gender::try_from(buffer.i32()?)?;
        let class_id = ClassId::try_from(buffer.i32()?)?;

        let mut primal_stats = PrimalStats::default();
        primal_stats.insert(PrimalStat::INT, buffer.u32()?);
        primal_stats.insert(PrimalStat::STR, buffer.u32()?);
        primal_stats.insert(PrimalStat::CON, buffer.u32()?);
        primal_stats.insert(PrimalStat::MEN, buffer.u32()?);
        primal_stats.insert(PrimalStat::DEX, buffer.u32()?);
        primal_stats.insert(PrimalStat::WIT, buffer.u32()?);

        let hair_style = buffer.u32()?;
        let hair_color = buffer.u32()?;

        let face = if buffer.remaining() >= 4 {
            buffer.u32()?
        } else {
            0
        };

        let appearance = character::Appearance {
            face,
            hair_style,
            hair_color,
            gender,
        };

        Ok(Self {
            name,
            race,
            class_id,
            primal_stats,
            appearance,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct RequestCharDelete(CharSlot);
impl RequestCharDelete {
    pub fn char_slot(&self) -> CharSlot {
        self.0
    }
}

impl TryFrom<ClientPacketBuffer> for RequestCharDelete {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        let char_slot = buffer.u32()?;
        Ok(Self(CharSlot(char_slot)))
    }
}
