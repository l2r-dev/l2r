use bevy::{
    ecs::{relationship::Relationship, system::ParallelCommands},
    prelude::*,
};
use game_core::{
    abnormal_effects::AbnormalEffects,
    attack::{AttackHit, Attacking, Dead, DeadTimer, DeathComponentsPlugin, InCombat},
    character::{Character, CharacterSave},
    custom_hierarchy::DespawnChildOf,
    network::{broadcast::ServerPacketBroadcast, packets::server::Die},
    npc::{GenerateDropRequest, NpcQuery},
    object_id::ObjectId,
    spawner::Spawner,
    stats::{ProgressLevelStats, ProgressRatesStats, ProgressStats, VitalsStat, VitalsStats},
};
use state::GameServerStateSystems;

pub struct DeathPlugin;
impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeathComponentsPlugin);

        app.add_systems(
            Update,
            (check_alive, dead_timer_handle).in_set(GameServerStateSystems::Run),
        );
        app.add_observer(death);
    }
}

pub fn dead_timer_handle(
    time: Res<Time>,
    mut query: Query<(Entity, Mut<DeadTimer>)>,
    par_commands: ParallelCommands,
) {
    let delta = time.delta();
    query.par_iter_mut().for_each(|(entity, mut timer)| {
        timer.tick(delta);
        if timer.finished() {
            par_commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            });
        }
    });
}

fn check_alive(
    par_commands: ParallelCommands,
    vitals: Query<(Entity, Ref<VitalsStats>), (With<Dead>, Changed<VitalsStats>)>,
) {
    vitals.par_iter().for_each(|(entity, vitals_stats)| {
        if vitals_stats.get(VitalsStat::Hp) > 0.0 {
            par_commands.command_scope(|mut commands| {
                commands.entity(entity).remove::<Dead>();
            });
        }
    });
}

fn death(
    death: Trigger<Dead>,
    mut commands: Commands,
    players: Query<Ref<ObjectId>, With<Character>>,
    mut progress_stats: Query<(
        Ref<ProgressLevelStats>,
        Ref<ProgressRatesStats>,
        Mut<ProgressStats>,
    )>,
    npcs: Query<(NpcQuery, Option<Ref<DespawnChildOf>>)>,
    mut spawners: Query<Mut<Spawner>>,
    mut abnormal_effects: Query<Mut<AbnormalEffects>>,
) {
    let entity = death.target();
    let event = death.event();
    let killer = event.killer();

    commands
        .entity(entity)
        .insert(*event)
        .remove::<(Attacking, AttackHit, InCombat)>();

    if let Ok(mut effects) = abnormal_effects.get_mut(entity) {
        effects.remove_all();
    }

    if let Ok(char_oid) = players.get(entity)
        && let Ok((p_level, _, mut p_stats)) = progress_stats.get_mut(entity)
    {
        // TODO: make exp loss to respect progress rates based
        // on who killed the character (pvp, pve, raid)
        p_stats.exp_lost(1.0, p_level.level());
        commands.trigger_targets(
            ServerPacketBroadcast::new(Die::new(*char_oid).to_village().into()),
            entity,
        );
        commands.trigger_targets(CharacterSave, entity);
    }

    if let Ok((npc, child_of)) = npcs.get(entity) {
        commands.entity(entity).try_insert(DeadTimer::default());
        commands.trigger_targets(
            ServerPacketBroadcast::new(Die::new(*npc.object_id).into()),
            entity,
        );

        commands.trigger_targets(GenerateDropRequest, entity);

        if let Some(child_of) = child_of
            && let Ok(mut spawner) = spawners.get_mut(child_of.get())
            && let Some(npc) = spawner.npc_mut(*npc.id)
        {
            npc.dec_count_alive();
        }

        if let Ok((_, p_rates, mut p_stats)) = progress_stats.get_mut(killer) {
            p_stats.add_exp(npc.progress_reward.exp, p_rates.exp_modifier().into());
            p_stats.add_sp(npc.progress_reward.sp, p_rates.sp_modifier().into());
        }
    }
}
