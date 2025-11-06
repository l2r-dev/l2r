use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, DollSlot, EquipItem, Inventory, ItemEquipped, ItemLocationVariant, ItemsDataAccess,
        ItemsDataQuery, ItemsDataQueryMut, Kind, PaperDoll, UnequipItem, UniqueItem, UpdateType,
    },
    network::packets::server::{
        GameServerPacket, InventoryUpdate, SendCharInfo, SendUserInfo, SystemMessage,
    },
    object_id::ObjectId,
    stats::{AttackEffects, StatModifiers, Weapon},
};
use l2r_core::db::{Repository, RepositoryManager, TypedRepositoryManager};
use sea_orm::{ActiveValue, IntoActiveModel};
use smallvec::smallvec;
use system_messages::{Id as SmId, SmParam};

pub struct EquipItemPlugin;
impl Plugin for EquipItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_equip_item);
        app.add_observer(handle_item_equipped);
    }
}

fn handle_equip_item(
    trigger: Trigger<EquipItem>,
    mut commands: Commands,
    mut characters: Query<(Mut<PaperDoll>, Mut<StatModifiers>, Ref<Inventory>)>,
    mut items_query: ItemsDataQueryMut,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().item_object_id;
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
        commands.trigger_targets(EquipItem::new(ammo_oid), character_entity);
    }

    let (doll_slot, previous_items) = paperdoll
        .equip(item_object_id, &items_query)
        .ok_or_else(|| BevyError::from("Unable to equip item"))?;

    if let Some(stats) = item_info.stats_modifiers() {
        stat_modifiers.merge(&stats);
    }

    for previous_oid in previous_items {
        commands.trigger_targets(UnequipItem::new(previous_oid), character_entity);
    }

    if let Ok(mut item) = items_query.item_by_object_id_mut(item_object_id) {
        item.equip(doll_slot);
    }

    commands.trigger_targets(ItemEquipped::new(item_object_id), character_entity);
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
    let item_object_id = trigger.event().item_object_id;
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
                item_active.location = ActiveValue::Set(ItemLocationVariant::PaperDoll);
                items_repository.update(&item_active).await?;
                Ok(())
            });
        }
        send_equipped_system_message(commands.reborrow(), character_entity, &unique_item);
    }
    commands.trigger_targets(SendUserInfo, character_entity);
    commands.trigger_targets(SendCharInfo, character_entity);
    Ok(())
}

fn send_equipped_system_message(
    mut commands: Commands,
    character_entity: Entity,
    unique_item: &UniqueItem,
) {
    if unique_item.item().enchant_level() > 0 {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::EquippedS1S2,
                vec![
                    SmParam::Number(unique_item.item().enchant_level().into()),
                    SmParam::Item(unique_item.item().id().into()),
                ],
            )),
            character_entity,
        );
    } else {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::YouHaveEquippedYourS1,
                vec![SmParam::Item(unique_item.item().id().into())],
            )),
            character_entity,
        );
    }
}
