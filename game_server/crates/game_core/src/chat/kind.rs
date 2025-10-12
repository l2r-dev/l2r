use bevy::prelude::Reflect;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::Display;

#[derive(
    Clone,
    Copy,
    Hash,
    Debug,
    Default,
    Display,
    IntoPrimitive,
    PartialEq,
    Eq,
    TryFromPrimitive,
    Reflect,
)]
#[repr(u32)]
pub enum Kind {
    #[default]
    General,
    Shout,
    Whisper,
    Party,
    Clan,
    Gm,
    PetitionPlayer,
    PetitionGm,
    Trade,
    Alliance,
    Announcement,
    Boat,
    Friend,
    MsnChat,
    PartyMatchRoom,
    PartyRoomCommander,
    PartyRoomAll,
    Hero,
    CriticalAnnounce,
    ScreenAnnounce,
    Battlefield,
    MpccRoom,
    NpcGeneral,
    NpcShout,
}

impl From<&[u8]> for Kind {
    fn from(item: &[u8]) -> Self {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(item);
        Kind::try_from(u32::from_le_bytes(bytes)).unwrap_or_default()
    }
}
