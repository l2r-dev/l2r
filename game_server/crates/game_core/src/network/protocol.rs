use bevy::prelude::Reflect;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
pub enum Version {
    #[default]
    Unknown = 0,
    HighFiveUpdate1 = 268,
    HighFiveUpdate2 = 271,
    HighFiveUpdate3 = 273,
}
impl Version {
    pub fn new(version: u32) -> Self {
        match version {
            268 => Self::HighFiveUpdate1,
            271 => Self::HighFiveUpdate2,
            273 => Self::HighFiveUpdate3,
            _ => Self::Unknown,
        }
    }
}

impl From<&[u8]> for Version {
    fn from(slice: &[u8]) -> Self {
        Self::new(u32::from_le_bytes(slice.try_into().unwrap_or_default()))
    }
}
