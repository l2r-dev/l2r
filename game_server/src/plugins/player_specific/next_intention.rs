use bevy::app::{App, FixedUpdate, Plugin};
use bevy_ecs::{
    change_detection::Ref,
    entity::Entity,
    prelude::{Has, IntoScheduleConfigs, ParallelCommands, Query, Without},
    query::{QueryData, QueryFilter},
};
use game_core::{
    action::{model::CoreAction, pickup::PickupRequest, wait_kind::Sit},
    animation::Animation,
    attack::{Attacking, Dead, InsertAttackingParams},
    object_id::ObjectId,
    path_finding::{InActionPathfindingTimer, VisibilityCheckRequest},
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
}

#[derive(QueryFilter)]
struct CharacterFilter {
    not_dead: Without<Dead>,
    not_animating: Without<Animation>,
    not_pathfinding: Without<InActionPathfindingTimer>,
}

fn next_intention_system(
    mut characters: Query<CharacterQuery, CharacterFilter>,
    par_commands: ParallelCommands,
) {
    characters.par_iter_mut().for_each(|character| {
        par_commands.command_scope(|mut commands| {
            commands.entity(character.entity).remove::<NextIntention>();

            match *character.next_intention {
                NextIntention::MoveTo { start, target } => {
                    if character.is_sitting {
                        return;
                    }

                    commands.trigger_targets(
                        VisibilityCheckRequest {
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

                    Attacking::insert(
                        &mut commands,
                        InsertAttackingParams {
                            attacker: character.entity,
                            target: target_entity,
                        },
                    );
                }
                NextIntention::PickUp { item } => {
                    if character.is_sitting {
                        return;
                    }

                    commands
                        .entity(character.entity)
                        .insert(PickupRequest(item));
                }
                NextIntention::CoreAction(v) => {
                    if character.is_sitting && v != CoreAction::SitStand {
                        return;
                    }

                    commands.trigger_targets(v, character.entity);
                }
                NextIntention::SpecialAction(v) => {
                    if character.is_sitting {
                        return;
                    }

                    commands.trigger_targets(v, character.entity);
                }
            }
        });
    });
}
