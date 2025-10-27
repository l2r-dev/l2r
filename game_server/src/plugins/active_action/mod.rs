use bevy::{ecs::system::ParallelCommands, prelude::*};
use game_core::active_action::{ActiveAction, ActiveActionComponentPlugin};

pub struct ActiveActionPlugin;
impl Plugin for ActiveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ActiveActionComponentPlugin);
        app.add_systems(FixedUpdate, active_action_timer_handler);
    }
}

fn active_action_timer_handler(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<ActiveAction>)>,
    par_commands: ParallelCommands,
) {
    let delta = time.delta();

    query
        .par_iter_mut()
        .for_each(|(entity, mut active_action)| {
            if active_action.proceed_timer(delta) {
                par_commands.command_scope(|mut commands| {
                    commands.entity(entity).remove::<ActiveAction>();
                });
            }
        });
}
