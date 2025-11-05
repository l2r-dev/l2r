use super::*;
use bevy::{
    ecs::{query::Has, system::SystemParam},
    log,
};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    attack::AttackHit,
    items::{
        BlessedSpiritshot, ConsumableKind, DollSlot, FishingShot, ItemsDataQueryMut, Kind,
        PaperDoll, ShotKind, Soulshot, Spiritshot, UseShot,
    },
    network::{
        broadcast::ServerPacketBroadcast,
        config::GameServerNetworkConfig,
        packets::{
            client::GameClientPacket,
            server::{AutoShotUse, GameServerPacket, MagicSkillUse, ResponseAutoShots, ShotState},
        },
        session::PacketReceiveParams,
    },
    skills::Skill,
};
use state::GameMechanicsSystems;
use std::time::Duration;

pub struct UseShotPlugin;
impl Plugin for UseShotPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoShotUse>();

        app.add_event::<UseShot>();

        app.add_observer(auto_shot_use);

        app.add_systems(
            FixedUpdate,
            (auto_shot_system, use_shot_handle)
                .chain()
                .in_set(GameMechanicsSystems::Items),
        );
    }
}

#[derive(SystemParam)]
struct ShotItemsParams<'w, 's> {
    items: Query<'w, 's, (Entity, Ref<'static, Item>)>,
    items_data: ItemsDataQuery<'w, 's>,
    object_id_manager: Res<'w, ObjectIdManager>,
}

#[derive(SystemParam)]
struct ShotStatusQueries<'w, 's> {
    soulshot_used: Query<'w, 's, Has<Soulshot>>,
    spiritshot_used: Query<'w, 's, Has<Spiritshot>>,
    blessed_spiritshot_used: Query<'w, 's, Has<BlessedSpiritshot>>,
    fishing_used: Query<'w, 's, Has<FishingShot>>,
}

impl ShotStatusQueries<'_, '_> {
    fn already_applied(&self, shot_kind: ShotKind, weapon_entity: Entity) -> bool {
        match shot_kind {
            ShotKind::Soulshot => self.soulshot_used.get(weapon_entity).unwrap_or(false),
            ShotKind::Spiritshot => self.spiritshot_used.get(weapon_entity).unwrap_or(false),
            ShotKind::BlessedSpiritshot => self
                .blessed_spiritshot_used
                .get(weapon_entity)
                .unwrap_or(false),
            ShotKind::Fishing => self.fishing_used.get(weapon_entity).unwrap_or(false),
        }
    }
}

#[derive(SystemParam)]
struct AutoShotUseParams<'w, 's> {
    receive_params: PacketReceiveParams<'w, 's>,
    commands: Commands<'w, 's>,
    character_inventories:
        Query<'w, 's, (Option<Ref<'static, AutoShotUse>>, Ref<'static, Inventory>)>,
    paper_dolls: Query<'w, 's, Ref<'static, PaperDoll>>,
    shot_items: ShotItemsParams<'w, 's>,
}

fn auto_shot_use(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    mut params: AutoShotUseParams,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::RequestAutoShots(ref packet) = event.packet else {
        return Ok(());
    };

    let entity = params.receive_params.character(&event.connection.id())?;
    let (auto_shot_use, inventory) = params.character_inventories.get(entity)?;

    if auto_shot_use.is_some() {
        params.commands.entity(entity).remove::<AutoShotUse>();
        params.commands.trigger_targets(
            GameServerPacket::from(ResponseAutoShots::new(packet.item_id, ShotState::Off)),
            entity,
        );
        return Ok(());
    }

    let paper_doll = params.paper_dolls.get(entity)?;

    // Use QueryLens to transform Query<(Entity, Ref<Item>)> to Query<Ref<Item>>
    let mut items_lens = params.shot_items.items.transmute_lens::<Ref<Item>>();
    let item_by_id = inventory.single_by_item_id(
        packet.item_id,
        &items_lens.query(),
        &params.shot_items.object_id_manager,
    );
    let Some((shot_entity, _shot_object_id, _shot_item_id)) = item_by_id else {
        return Ok(());
    };

    let Ok((shot_entity, shot_item)) = params.shot_items.items.get(shot_entity) else {
        log::warn!("No shot item found for entity {:?}", shot_entity);
        return Ok(());
    };

    // Check weapon compatibility with shot
    let compatibility =
        check_weapon_shot_compatibility(shot_entity, &paper_doll, &params.shot_items.items_data)?;
    let Some(compatibility) = compatibility else {
        log::warn!("Weapon shot compatibility check failed");
        return Ok(());
    };

    log::info!(
        "Auto shot activation successful for with shot {:?}",
        compatibility.shot_kind
    );

    params
        .commands
        .entity(entity)
        .try_insert(AutoShotUse::from(shot_item.id()));

    params.commands.trigger_targets(
        GameServerPacket::from(ResponseAutoShots::new(shot_item.id(), ShotState::On)),
        entity,
    );

    Ok(())
}

#[derive(SystemParam)]
struct AutoShotSystemParams<'w, 's> {
    commands: Commands<'w, 's>,
    auto_shot_characters: Query<
        'w,
        's,
        (
            Entity,
            Ref<'static, AutoShotUse>,
            Ref<'static, PaperDoll>,
            Ref<'static, Inventory>,
        ),
    >,
    shot_items: ShotItemsParams<'w, 's>,
    shot_status: ShotStatusQueries<'w, 's>,
    use_shot_events: EventWriter<'w, UseShot>,
}

#[derive(SystemParam)]
struct UseShotHandleParams<'w, 's> {
    events: EventReader<'w, 's, UseShot>,
    character_query: Query<
        'w,
        's,
        (
            Ref<'static, ObjectId>,
            Ref<'static, PaperDoll>,
            Ref<'static, Inventory>,
            Ref<'static, Transform>,
        ),
        Without<AttackHit>,
    >,
    commands: Commands<'w, 's>,
    items_data: ItemsDataQueryMut<'w, 's>,
    item_objects: Query<'w, 's, Ref<'static, ObjectId>, With<Item>>,
    shot_status: ShotStatusQueries<'w, 's>,
}

fn auto_shot_system(mut params: AutoShotSystemParams) -> Result<()> {
    for (entity, auto_shot_use, paper_doll, inventory) in params.auto_shot_characters.iter() {
        let mut items_lens = params.shot_items.items.transmute_lens::<Ref<Item>>();
        let item_by_id = inventory.single_by_item_id(
            **auto_shot_use,
            &items_lens.query(),
            &params.shot_items.object_id_manager,
        );
        let Some((shot_entity, _shot_object_id, _shot_item_id)) = item_by_id else {
            params.commands.entity(entity).remove::<AutoShotUse>();
            params.commands.trigger_targets(
                GameServerPacket::from(ResponseAutoShots::new(**auto_shot_use, ShotState::Off)),
                entity,
            );
            continue;
        };

        let Ok((shot_entity, _)) = params.shot_items.items.get(shot_entity) else {
            log::warn!("No shot item found for entity {:?}", shot_entity);
            continue;
        };

        let compatibility = check_weapon_shot_compatibility(
            shot_entity,
            &paper_doll,
            &params.shot_items.items_data,
        )?;

        let Some(compatibility) = compatibility else {
            params.commands.entity(entity).remove::<AutoShotUse>();
            params.commands.trigger_targets(
                GameServerPacket::from(ResponseAutoShots::new(**auto_shot_use, ShotState::Off)),
                entity,
            );
            continue;
        };

        let weapon_entity = params
            .shot_items
            .items
            .by_object_id(compatibility.oid, &params.shot_items.object_id_manager)?
            .0;
        if !params
            .shot_status
            .already_applied(compatibility.shot_kind, weapon_entity)
        {
            params
                .use_shot_events
                .write(UseShot::new(entity, shot_entity));
        }
    }

    Ok(())
}

fn use_shot_handle(mut params: UseShotHandleParams) -> Result<()> {
    for (event, _event_id) in params.events.par_read() {
        let used_entity = event.entity();
        let Ok((char_oid, paper_doll, inventory, transform)) =
            params.character_query.get(used_entity)
        else {
            continue;
        };

        // First, collect all the data we need from the immutable query
        let (shot_item_id, compatibility, weapon_entity) = {
            let shot_entity = event.shot_entity();

            let compatibility =
                check_weapon_shot_compatibility(shot_entity, &paper_doll, &params.items_data)?;

            let shot_item_id = params.items_data.item(shot_entity)?.id();

            let Some(compatibility) = compatibility else {
                params.commands.trigger_targets(
                    GameServerPacket::from(ResponseAutoShots::new(
                        shot_item_id,
                        ShotState::default(),
                    )),
                    used_entity,
                );
                // TODO: Send 'Can not use item' packet to client
                continue;
            };

            let weapon_entity = params.items_data.entity(compatibility.oid)?;

            (shot_item_id, compatibility, weapon_entity)
        };
        let shot_kind = compatibility.shot_kind;
        let shot_count = compatibility.shot_count;

        if params.shot_status.already_applied(shot_kind, weapon_entity) {
            log::warn!(
                "{:?} already used for entity {:?}",
                shot_kind,
                weapon_entity
            );
            continue;
        }

        let Ok(shot_object_id) = params.item_objects.get(event.shot_entity()) else {
            log::warn!("No object id found for item {:?}", event.shot_entity());
            continue;
        };

        if inventory.get_item(*shot_object_id).is_err() {
            continue;
        }

        let mut item = params.items_data.item_by_object_id_mut(*shot_object_id)?;

        let current_count = item.count();
        if current_count < shot_count as u64 {
            log::warn!(
                "Not enough shots available: {} < {}",
                current_count,
                shot_count
            );
            continue;
        }
        item.set_count(current_count - shot_count as u64);

        match shot_kind {
            ShotKind::Soulshot => {
                params.commands.entity(weapon_entity).insert(Soulshot);
            }
            ShotKind::Spiritshot => {
                params.commands.entity(weapon_entity).insert(Spiritshot);
            }
            ShotKind::BlessedSpiritshot => {
                params
                    .commands
                    .entity(weapon_entity)
                    .insert(BlessedSpiritshot);
            }
            ShotKind::Fishing => {
                params.commands.entity(weapon_entity).insert(FishingShot);
            }
        }

        let shot_item_info = params.items_data.item_info(shot_item_id)?;
        let Some(item_skills) = shot_item_info.item_skills() else {
            log::warn!("No skills found for item {:?}", event.shot_entity());
            continue;
        };

        // get first, cause shot can have only one skill
        let Some(item_skill) = item_skills.first() else {
            log::warn!("No skills found for item {:?}", event.shot_entity());
            continue;
        };

        let skill: Skill = (*item_skill).into();

        let magic_use = MagicSkillUse::new(
            *char_oid,
            transform.translation,
            *char_oid,
            transform.translation,
            skill,
            Duration::from_millis(0),
            Duration::from_millis(0),
        );

        params
            .commands
            .trigger_targets(ServerPacketBroadcast::new(magic_use.into()), event.entity());
    }
    Ok(())
}

#[derive(Debug)]
struct WeaponShotCompatibility {
    oid: ObjectId,
    shot_kind: ShotKind,
    shot_count: u32,
}

fn check_weapon_shot_compatibility(
    shot_entity: Entity,
    paper_doll: &PaperDoll,
    items_data: &impl ItemsDataAccess,
) -> Result<Option<WeaponShotCompatibility>> {
    // Check if weapon is equipped
    let weapon = paper_doll.get(DollSlot::RightHand);
    let Some(weapon_oid) = weapon else {
        return Ok(None);
    };

    let weapon_item_info = items_data.info_by_object_id(weapon_oid)?;
    let shot_item = items_data.item(shot_entity)?;
    let shot_item_info = items_data.item_info(shot_item.id())?;

    // Check grade compatibility
    let weapon_grade = weapon_item_info.grade().shot_grade();
    let shot_grade = shot_item_info.grade();
    if weapon_grade != shot_grade {
        return Ok(None);
    }

    let (shot_kind, shot_count) = match weapon_item_info.kind() {
        Kind::Weapon(weapon) => match shot_item_info.kind() {
            Kind::Consumable(ConsumableKind::Shot(shot_kind)) => match shot_kind {
                ShotKind::Soulshot => (shot_kind, weapon.soulshots),
                ShotKind::Spiritshot | ShotKind::BlessedSpiritshot => {
                    (shot_kind, weapon.spiritshots)
                }
                ShotKind::Fishing => (shot_kind, weapon.soulshots),
            },
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };

    if shot_count == 0 {
        return Ok(None);
    }

    Ok(Some(WeaponShotCompatibility {
        oid: weapon_oid,
        shot_kind: *shot_kind,
        shot_count,
    }))
}
