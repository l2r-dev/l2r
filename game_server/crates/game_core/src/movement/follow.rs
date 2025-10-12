use bevy::prelude::*;
use derive_more::derive::From;
use smallvec::SmallVec;

pub struct FollowComponentsPlugin;
impl Plugin for FollowComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Following>()
            .register_type::<Followers>();

        app.add_event::<FollowRequest>();
    }
}

#[derive(Deref, Event, From)]
pub struct FollowRequest(Entity);

const FOLLOWERS_CAPACITY: usize = 10;

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = Following)]
pub struct Followers(SmallVec<[Entity; FOLLOWERS_CAPACITY]>);

#[derive(Clone, Component, Copy, Debug, Deref, PartialEq, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Followers)]
pub struct Following(pub Entity);
