use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(
    Display, Default, Copy, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect,
)]
#[repr(u8)]
pub enum EtcKind {
    #[default]
    None,
    Usable,
    Alphabet,
    Quest,
    Coin,
    Coupon,
    CastleGuard,
    Dye,
    Lotto,
    LifeCrystal,
    Lure,
    Map,
    Talisman,
    Ticket,
    Rune,
    Spellbook,
}

impl From<EtcKind> for u32 {
    fn from(value: EtcKind) -> Self {
        value as u32
    }
}

impl From<u32> for EtcKind {
    fn from(value: u32) -> Self {
        match value {
            0 => EtcKind::None,
            1 => EtcKind::Usable,
            2 => EtcKind::Alphabet,
            3 => EtcKind::Quest,
            4 => EtcKind::Coin,
            5 => EtcKind::Coupon,
            6 => EtcKind::CastleGuard,
            7 => EtcKind::Dye,
            8 => EtcKind::Lotto,
            9 => EtcKind::LifeCrystal,
            10 => EtcKind::Lure,
            11 => EtcKind::Map,
            12 => EtcKind::Talisman,
            13 => EtcKind::Ticket,
            14 => EtcKind::Rune,
            15 => EtcKind::Spellbook,
            _ => EtcKind::None,
        }
    }
}
