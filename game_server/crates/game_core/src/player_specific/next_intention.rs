use crate::action::model::{CoreAction, SpecialAction};
use bevy::{
    app::{App, Plugin},
    math::Vec3,
    prelude::Reflect,
};
use bevy_ecs::{
    entity::Entity,
    prelude::{Component, ReflectComponent},
};

pub struct NextIntentionComponentsPlugin;

impl Plugin for NextIntentionComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NextIntention>();
    }
}

#[derive(Clone, Component, Copy, Debug, Reflect)]
#[reflect(Component)]
pub enum NextIntention {
    MoveTo { start: Vec3, target: Vec3 },
    Attack { target: Entity },
    CoreAction(CoreAction),
    SpecialAction(SpecialAction),
    PickUp { item: Entity },
    Follow { target: Entity },
    DialogRequest { target: Entity },
}
