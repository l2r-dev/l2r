use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    active_action::ActiveAction,
    items::{
        self, DollSlot, EquipItem, Inventory, ItemEquipped, ItemEquippedMessage, ItemsDataAccess,
        ItemsDataQuery, ItemsDataQueryMut, Kind, PaperDoll, UnequipItem, UniqueItem, UpdateType,
        model::ActiveModelSetCoordinates,
    },
    network::packets::server::{
        BroadcastCharInfo, GameServerPacket, InventoryUpdate, SendUserInfo, SystemMessage,
    },
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
    mut characters: Query<
        (Mut<PaperDoll>, Mut<StatModifiers>, Ref<Inventory>),
        Without<ActiveAction>,
    >,
    mut items_query: ItemsDataQueryMut,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().0;
    let (mut paperdoll, mut stat_modifiers, inventory) = characters.get_mut(character_entity)?;

    let item = items_query.item_by_object_id(item_object_id)?;
    let item_info = items_query.item_info(item.id())?;

    // Validate ammo compatible with weapon in right hand
    if item_info.kind().ammo() {
        let valid_ammo = paperdoll[DollSlot::RightHand]
            .and_then(|rh_oid| items_query.info_by_object_id(rh_oid).ok())
            .map(|rh_info| rh_info.ammo_matches(item_info))
            .unwrap_or(false);

        if !valid_ammo {
            return Ok(());
        }
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

    let (doll_slot, previous_items) = paperdoll
        .equip(item_object_id, &items_query)
        .ok_or_else(|| BevyError::from("Unable to equip item"))?;

    if let Some(stats) = item_info.stats_modifiers() {
        stat_modifiers.merge(&stats);
    }

    for previous_oid in previous_items {
        commands.trigger_targets(
            UnequipItem {
                item_object_id: previous_oid,
                skip_db_update: false,
            },
            character_entity,
        );
    }

    if let Ok(mut item) = items_query.item_by_object_id_mut(item_object_id) {
        item.equip(doll_slot);
    }

    commands.trigger_targets(ItemEquipped(item_object_id), character_entity);
    Ok(())
}

fn handle_item_equipped(
    trigger: Trigger<ItemEquipped>,
    mut commands: Commands,
    repo_manager: Res<RepositoryManager>,
    items_query: ItemsDataQuery,
    mut attack_effects: Query<Mut<AttackEffects>>,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().0;
    let item_info = items_query.info_by_object_id(item_object_id)?;

    // Update attack effects if this is a weapon
    if let Ok(mut attack_effects) = attack_effects.get_mut(character_entity)
        && let Kind::Weapon(weapon) = item_info.kind()
    {
        let weapon_effect = Weapon::from(weapon.kind);
        attack_effects.set_weapon(weapon_effect);
    }

    // Get item data for client update
    let unique_item = items_query
        .item_by_object_id(item_object_id)
        .ok()
        .map(|item| UniqueItem::new(item_object_id, *item));

    if let Some(unique_item) = unique_item {
        let inventory_update = InventoryUpdate::new(smallvec![unique_item], UpdateType::Modify);
        commands.trigger_targets(GameServerPacket::from(inventory_update), character_entity);

        if !repo_manager.is_mock() {
            let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
            let item_model = items::model::Model::from(unique_item);
            commands.spawn_task(move || async move {
                let mut item_active = item_model.into_active_model();
                item_active.set_location(unique_item.item().location());
                items_repository.update(&item_active).await?;
                Ok(())
            });
        }
        commands.trigger_targets(ItemEquippedMessage(*unique_item.item()), character_entity);
    }
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
