use bevy::prelude::*;
use derive_more::{From, Into};
use l2r_core::packets::ServerPacketBuffer;

#[derive(Clone, Copy, Debug, Default, Deref, From, Into, PartialEq, Reflect)]
pub struct SysStringId(u32);

#[derive(Clone, Debug, PartialEq, Reflect)]
#[repr(u32)]
pub enum SmParam {
    Text(String),
    Number(u32),
    Npc(u32),
    Item(u32),
    Skill((u32, u32)), // (id, level)
    Castle(u8),
    LongNumber(u64),
    Zone(Vec3),
    ItemName2(u32),
    Element(u8),
    Instance(u32),
    Door(u32),
    Player(String),
    SystemString(SysStringId),
}

impl SmParam {
    fn discriminant(&self) -> u32 {
        // https://doc.rust-lang.org/reference/items/enumerations.html#r-items.enum.discriminant.access-memory
        unsafe { *(self as *const Self as *const u32) }
    }
    pub fn to_le_bytes(&self) -> ServerPacketBuffer {
        use self::SmParam::*;

        let mut buffer = ServerPacketBuffer::new();

        buffer.u32(self.discriminant());

        match self {
            Text(text) => {
                buffer.str(text);
            }
            Number(num) => {
                buffer.u32(*num);
            }
            Npc(object_id) => {
                buffer.u32(*object_id);
            }
            Item(item_id) => {
                buffer.u32(*item_id);
            }
            Skill(skill) => {
                buffer.u32(skill.0);
                buffer.u32(skill.1);
            }
            Castle(castle_id) => {
                buffer.u8(*castle_id);
            }
            LongNumber(num) => {
                buffer.u64(*num);
            }
            Zone(vec) => {
                let mut bytes = [0; 12];
                // Y in bevy is up, in l2 Z is up
                bytes[0..4].copy_from_slice(&vec.x.to_le_bytes());
                bytes[4..8].copy_from_slice(&vec.z.to_le_bytes());
                bytes[8..12].copy_from_slice(&vec.y.to_le_bytes());
                buffer.extend(bytes);
            }
            ItemName2(item_id) => {
                buffer.u32(*item_id);
            }
            Element(element) => {
                buffer.u8(*element);
            }
            Instance(instance_id) => {
                buffer.u32(*instance_id);
            }
            Door(door_id) => {
                buffer.u32(*door_id);
            }
            Player(player_name) => {
                buffer.str(player_name);
            }
            SystemString(sys_string_id) => {
                buffer.u32((*sys_string_id).into());
            }
        }
        buffer
    }
}

impl From<&str> for SmParam {
    fn from(text: &str) -> Self {
        SmParam::Text(text.to_string())
    }
}
impl From<String> for SmParam {
    fn from(text: String) -> Self {
        SmParam::Text(text)
    }
}
impl From<u32> for SmParam {
    fn from(num: u32) -> Self {
        SmParam::Number(num)
    }
}

impl From<Vec3> for SmParam {
    fn from(vec: Vec3) -> Self {
        SmParam::Zone(vec)
    }
}

impl From<u64> for SmParam {
    fn from(num: u64) -> Self {
        SmParam::LongNumber(num)
    }
}
