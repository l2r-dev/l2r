use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    attack::Attacking,
    items::{model as items_model, *},
    network::packets::server::*,
    object_id::ObjectId,
    stats::{AttackEffects, StatModifiers, Weapon},
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use sea_orm::IntoActiveModel;
use smallvec::smallvec;
use system_messages::{Id as SmId, SmParam};

pub struct EquipItemPlugin;
impl Plugin for EquipItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_equip_item);
        app.add_observer(handle_item_equipped);
        app.add_observer(send_equipped_system_message);
    }
}

fn handle_equip_item(
    trigger: Trigger<EquipItem>,
    mut commands: Commands,
    mut characters: Query<(InventoriesQueryMut, Mut<StatModifiers>), Without<Attacking>>,
    mut items_query: ItemsDataQueryMut,
    repo_manager: Res<RepositoryManager>,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().0;

    let (InventoriesQueryMutReadOnlyItem { paper_doll, .. }, _) =
        characters.get(character_entity)?;
    let Some((equip_slot, unequip_slots)) = paper_doll.desired_slot(item_object_id, &items_query)
    else {
        return Ok(());
    };

    for unequip_slot in unequip_slots {
        let equipped_oid = {
            let (InventoriesQueryMutReadOnlyItem { paper_doll, .. }, _) =
                characters.get(character_entity)?;
            paper_doll.get(unequip_slot)
        };
        if let Some(equipped_oid) = equipped_oid {
            let mut dolls_lens = characters.transmute_lens::<Mut<PaperDoll>>();
            let doll_query = dolls_lens.query();
            super::process_unequip(
                character_entity,
                equipped_oid,
                commands.reborrow(),
                &mut items_query,
                doll_query,
                false,
                &repo_manager,
            )?;
        }
    }

    let (
        InventoriesQueryMutReadOnlyItem {
            paper_doll,
            inventory,
            ..
        },
        _,
    ) = characters.get(character_entity)?;

    let item = items_query.item_by_object_id(item_object_id)?;
    let item_info = items_query.item_info(item.id())?;

    if item_info.kind().ammo() && !paper_doll.is_ammo_valid_for_weapon(item_object_id, &items_query)
    {
        return Ok(());
    }

    // Auto-equip matching ammo when equipping a bow/crossbow
    if item_info.kind().bow_or_crossbow()
        && let Some(ammo_oid) = inventory
            .iter()
            .find(|&&ammo_oid| {
                items_query
                    .info_by_object_id(ammo_oid)
                    .is_ok_and(|ammo_info| item_info.ammo_matches(ammo_info))
            })
            .copied()
    {
        commands.trigger_targets(EquipItem(ammo_oid), character_entity);
    }

    let (InventoriesQueryMutItem { mut paper_doll, .. }, mut stat_modifiers) =
        characters.get_mut(character_entity)?;

    paper_doll.equip_slot(equip_slot, item_object_id);

    if let Some(stats) = item_info.stats_modifiers() {
        stat_modifiers.merge(&stats);
    }

    if let Ok(mut item) = items_query.item_by_object_id_mut(item_object_id) {
        item.equip(equip_slot);
    }

    let unique_item = items_query
        .item_by_object_id(item_object_id)
        .ok()
        .map(|item| UniqueItem::new(item_object_id, *item));

    if let Some(unique_item) = unique_item {
        let inventory_update = InventoryUpdate::new(smallvec![unique_item], UpdateType::Modify);
        commands.trigger_targets(GameServerPacket::from(inventory_update), character_entity);

        if !repo_manager.is_mock() {
            let items_repository = repo_manager.typed::<ObjectId, items_model::Entity>()?;
            let item_model = items_model::Model::from(unique_item);
            commands.spawn_task(move || async move {
                let mut item_active = item_model.into_active_model();
                items_model::ActiveModelSetCoordinates::set_location(
                    &mut item_active,
                    unique_item.item().location(),
                );
                items_repository.update(&item_active).await?;
                Ok(())
            });
        }
    }

    commands.trigger_targets(
        ItemEquipped {
            item_object_id,
            slot: equip_slot,
        },
        character_entity,
    );

    Ok(())
}

fn handle_item_equipped(
    trigger: Trigger<ItemEquipped>,
    mut commands: Commands,
    items_query: ItemsDataQuery,
    mut attack_effects: Query<Mut<AttackEffects>>,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().item_object_id;
    let item_info = items_query.info_by_object_id(item_object_id)?;

    // TODO: Send Update stats events
    if let Ok(mut attack_effects) = attack_effects.get_mut(character_entity)
        && let Kind::Weapon(weapon) = item_info.kind()
    {
        let weapon_effect = Weapon::from(weapon.kind);
        attack_effects.set_weapon(weapon_effect);
    }

    let item = items_query.item_by_object_id(item_object_id)?;
    commands.trigger_targets(ItemEquippedMessage(*item), character_entity);
    commands.trigger_targets(SendUserInfo, character_entity);
    commands.trigger_targets(BroadcastCharInfo, character_entity);
    Ok(())
}

pub fn send_equipped_system_message(trigger: Trigger<ItemEquippedMessage>, mut commands: Commands) {
    let item = trigger.event().0;
    let entity = trigger.target();
    if item.enchant_level() > 0 {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::EquippedS1S2,
                vec![
                    SmParam::Number(item.enchant_level().into()),
                    SmParam::Item(item.id().into()),
                ],
            )),
            entity,
        );
    } else {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::YouHaveEquippedYourS1,
                vec![SmParam::Item(item.id().into())],
            )),
            entity,
        );
    }
}
