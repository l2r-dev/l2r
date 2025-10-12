use super::GameServerPacketCodes;
use crate::object_id::ObjectId;
use bevy::prelude::*;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};
use num_enum::IntoPrimitive;
use std::{fmt, time::Duration};

#[repr(u32)]
#[derive(Clone, Copy, Debug, IntoPrimitive, PartialEq, Reflect)]
pub enum SetupGaugeColor {
    Blue,
    Red,
    Cyan,
    Green,
}

#[derive(Clone, Reflect)]
pub struct SetupGauge {
    object_id: ObjectId,
    color: SetupGaugeColor,
    current_time: Duration,
    total_time: Duration,
}
impl fmt::Debug for SetupGauge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SetupGauge {{ object_id: {:?}, color: {:?}, current_time: {:?}, total_time: {:?} }}",
            self.object_id,
            self.color,
            self.current_time.as_millis(),
            self.total_time.as_millis()
        )
    }
}

impl L2rServerPacket for SetupGauge {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(GameServerPacketCodes::SETUP_GAUGE.to_le_bytes());
        buffer.u32(self.object_id.into());
        buffer.u32(self.color.into());
        buffer.u32(self.current_time.as_millis() as u32);
        buffer.u32(self.total_time.as_millis() as u32);
        buffer
    }
}
impl SetupGauge {
    pub fn new(object_id: ObjectId, color: SetupGaugeColor, total_time: Duration) -> Self {
        SetupGauge {
            object_id,
            color,
            current_time: total_time,
            total_time,
        }
    }

    pub fn new_with_current(
        object_id: ObjectId,
        color: SetupGaugeColor,
        current_time: Duration,
        total_time: Duration,
    ) -> Self {
        SetupGauge {
            object_id,
            color,
            current_time,
            total_time,
        }
    }
}
