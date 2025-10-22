use bevy::{ecs::system::ParallelCommands, prelude::*};
use game_core::animation::{Animation, AnimationComponentPlugin};

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationComponentPlugin);
        app.add_systems(FixedUpdate, animation_timer_handler);
    }
}

fn animation_timer_handler(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<Animation>)>,
    par_commands: ParallelCommands,
) {
    let delta = time.delta();

    query.par_iter_mut().for_each(|(entity, mut animation)| {
        if animation.proceed_timer(delta) {
            par_commands.command_scope(|mut commands| {
                commands.entity(entity).remove::<Animation>();
            });
        }
    });
}
