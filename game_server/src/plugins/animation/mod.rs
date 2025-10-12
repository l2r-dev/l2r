use bevy::{ecs::system::ParallelCommands, prelude::*};
use game_core::animation::{Animation, AnimationComponentPlugin, AnimationTimer};

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationComponentPlugin);
        app.add_systems(FixedUpdate, animation_timer_handler);
    }
}

fn animation_timer_handler(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<AnimationTimer>)>,
    par_commands: ParallelCommands,
) {
    let delta = time.delta();
    query.par_iter_mut().for_each(|(entity, mut timer)| {
        timer.tick(delta);
        if timer.just_finished() {
            par_commands.command_scope(|mut commands| {
                commands
                    .entity(entity)
                    .remove::<(AnimationTimer, Animation)>();
            });
        }
    });
}
