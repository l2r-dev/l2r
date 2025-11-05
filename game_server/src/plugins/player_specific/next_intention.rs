use bevy::app::{App, FixedUpdate, Plugin};
use bevy_ecs::{
    change_detection::Ref,
    entity::Entity,
    prelude::{Has, IntoScheduleConfigs, ParallelCommands, Query, Without},
    query::QueryData,
};
use game_core::{
    action::{model::CoreAction, pickup::PickupRequest, wait_kind::Sit},
    active_action::ActiveAction,
    attack::{Attacking, Dead},
    movement::{FollowRequest, Following},
    npc::DialogRequest,
    object_id::ObjectId,
    path_finding::DirectMoveRequest,
    player_specific::next_intention::{NextIntention, NextIntentionComponentsPlugin},
};
use state::GameMechanicsSystems;

pub struct NextIntentionPlugin;

impl Plugin for NextIntentionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NextIntentionComponentsPlugin);

        app.add_systems(
            FixedUpdate,
            next_intention_system.in_set(GameMechanicsSystems::NextIntention),
        );
    }
}

#[derive(QueryData)]
struct CharacterQuery<'a> {
    entity: Entity,
    object_id: Ref<'a, ObjectId>,
    next_intention: Ref<'a, NextIntention>,
    is_sitting: Has<Sit>,
    is_dead: Has<Dead>,
}

fn next_intention_system(
    mut characters: Query<CharacterQuery, Without<ActiveAction>>,
    par_commands: ParallelCommands,
) {
    characters.par_iter_mut().for_each(|character| {
        par_commands.command_scope(|mut commands| {
            if character.is_dead {
                commands.entity(character.entity).remove::<NextIntention>();

                return;
            }

            commands.entity(character.entity).remove::<NextIntention>();

            match *character.next_intention {
                NextIntention::MoveTo { start, target } => {
                    if character.is_sitting {
                        return;
                    }

                    commands.entity(character.entity).remove::<(
                        PickupRequest,
                        Following,
                        Attacking,
                        DialogRequest,
                    )>();

                    commands.trigger_targets(
                        DirectMoveRequest {
                            entity: character.entity,
                            start,
                            target,
                        },
                        character.entity,
                    );
                }
                NextIntention::Attack {
                    target: target_entity,
                } => {
                    if character.is_sitting {
                        return;
                    }

                    commands
                        .entity(character.entity)
                        .insert(Attacking(target_entity));
                }
                NextIntention::PickUp { item } => {
                    if character.is_sitting {
                        return;
                    }

                    commands
                        .entity(character.entity)
                        .insert(PickupRequest(item));
                }
                NextIntention::CoreAction(core_action) => {
                    if character.is_sitting && core_action != CoreAction::SitStand {
                        return;
                    }

                    commands.trigger_targets(core_action, character.entity);
                }
                NextIntention::SpecialAction(special_action) => {
                    if character.is_sitting {
                        return;
                    }

                    commands.trigger_targets(special_action, character.entity);
                }
                NextIntention::Follow { target } => {
                    if character.is_sitting {
                        return;
                    }

                    commands.trigger_targets(FollowRequest::from(target), character.entity);
                }
                NextIntention::DialogRequest { target } => {
                    commands
                        .entity(character.entity)
                        .insert(DialogRequest::from(target));
                }
            }
        });
    });
}
