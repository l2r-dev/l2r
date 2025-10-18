use bevy::prelude::*;
use num_enum::IntoPrimitive;
use serde::Deserialize;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, IntoPrimitive, PartialEq, Reflect)]
pub enum BodyPart {
    #[default]
    None = 0x0000,
    Underwear = 0x0001,
    RightEar = 0x0002,
    LeftEar = 0x0004,
    BothEar = 0x0006,
    Neck = 0x0008,
    RightFinger = 0x0010,
    LeftFinger = 0x0020,
    BothFinger = 0x0030,
    Head = 0x0040,
    RightHand = 0x0080,
    LeftHand = 0x0100,
    Gloves = 0x0200,
    Chest = 0x0400,
    Legs = 0x0800,
    Feet = 0x1000,
    Back = 0x2000,
    BothHand = 0x4000,
    FullBody = 0x8000,
    AccessoryLeft = 0x010000,
    AllDress = 0x020000,
    AccessoryRight = 0x040000,
    AccessoryBoth = 0x080000,
    RightBracelet = 0x100000,
    LeftBracelet = 0x200000,
    Talisman = 0x400000,
    Belt = 0x10000000,
}
