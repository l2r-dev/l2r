use bevy::prelude::*;
use num_enum::IntoPrimitive;

#[derive(Clone, Copy, Debug, Default, IntoPrimitive, Reflect)]
#[repr(u8)]
pub enum ShieldResult {
    #[default]
    Failed,
    Succeed,
    PerfectBlock,
}

impl From<ShieldResult> for i64 {
    fn from(val: ShieldResult) -> Self {
        (val as u8).into()
    }
}
