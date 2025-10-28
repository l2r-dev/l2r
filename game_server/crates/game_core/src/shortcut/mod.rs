use super::{network::packets::client::RequestShortcutRegistration, object_id::ObjectId};
use crate::stats::SubClassVariant;
use bevy::prelude::*;
use l2r_core::packets::ServerPacketBuffer;
use num_enum::{IntoPrimitive, TryFromPrimitive};

mod kind;
pub mod model;
mod slot_id;

pub use kind::*;
pub use slot_id::*;

pub struct ShortcutComponentsPlugin;
impl Plugin for ShortcutComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShortcutKind>()
            .register_type::<ShortcutTargetKind>()
            .register_type::<Shortcut>()
            .register_type::<model::Model>();
    }
}

#[derive(Clone, Copy, Debug, Default, IntoPrimitive, PartialEq, Reflect, TryFromPrimitive)]
#[repr(u32)]
pub enum ShortcutTargetKind {
    #[default]
    Character = 1,
    Pet,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Shortcut {
    pub class_variant: SubClassVariant,
    pub kind: ShortcutKind,
    pub target: ShortcutTargetKind,
    pub reuse_group: i32,
    pub slot_id: SlotId,
}

impl Shortcut {
    pub fn from_packet(
        class_variant: SubClassVariant,
        packet: &RequestShortcutRegistration,
    ) -> Self {
        Self {
            class_variant,
            slot_id: packet.slot_id,
            kind: packet.kind,
            target: packet.target,
            reuse_group: -1,
        }
    }

    pub fn into_model(self, char_id: ObjectId) -> model::Model {
        let mut level: Option<i32> = None;
        let shortcut_id: i32 = match self.kind {
            ShortcutKind::Item(object_id) => i32::from(object_id),
            ShortcutKind::Skill(skill_id, skill_level) => {
                level = Some(skill_level.into());
                skill_id.into()
            }
            ShortcutKind::Action(action_id) => action_id.into(),
            ShortcutKind::Macro(macro_id) => macro_id as i32,
            ShortcutKind::Recipe(recipe_id) => recipe_id as i32,
            ShortcutKind::Bookmark(bookmark_id) => bookmark_id as i32,
            _ => 0,
        };

        model::Model {
            char_id,
            slot_id: self.slot_id,
            class_variant: self.class_variant,
            kind: self.kind.into(),
            shortcut_id,
            level,
        }
    }

    pub fn into_buffer(&self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::default();
        buffer.u32(self.kind.variant() as u32);
        buffer.u32(self.slot_id.into());
        match self.kind {
            ShortcutKind::Item(object_id) => {
                buffer.u32(object_id.into());
                buffer.u32(self.target.into());
                buffer.i32(self.reuse_group);
                buffer.u32(0);
                buffer.u32(0);
                buffer.u32(0); // item augment id
            }
            ShortcutKind::Skill(skill_id, skill_level) => {
                buffer.u32(skill_id.into());
                buffer.u32(skill_level.into());
                buffer.u8(0);
                buffer.u32(self.target as u32);
            }
            ShortcutKind::Action(action_id) => {
                buffer.u32(action_id.into());
                buffer.u32(self.target as u32);
            }
            ShortcutKind::Macro(macro_id) => {
                buffer.u32(macro_id);
                buffer.u32(self.target as u32);
            }
            ShortcutKind::Recipe(recipe_id) => {
                buffer.u32(recipe_id);
                buffer.u32(self.target as u32);
            }
            ShortcutKind::Bookmark(bookmark_id) => {
                buffer.u32(bookmark_id);
                buffer.u32(self.target as u32);
            }
            ShortcutKind::None => {}
        }
        buffer
    }
}

impl From<model::Model> for Shortcut {
    fn from(model: model::Model) -> Self {
        Self {
            slot_id: model.slot_id,
            class_variant: model.class_variant,
            kind: ShortcutKind::new(
                model.kind,
                model.shortcut_id as u32,
                model.level.unwrap_or(1).into(),
            )
            .unwrap_or_default(),
            target: ShortcutTargetKind::Character,
            reuse_group: -1,
        }
    }
}
