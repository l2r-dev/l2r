use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, DollSlot, Item, ItemLocationVariant, ItemUnequipped, ItemsDataAccess, ItemsDataQuery,
        ItemsDataQueryMut, PaperDoll, UnequipItem, UniqueItem, UpdateType,
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

pub struct UnequipItemPlugin;
impl Plugin for UnequipItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_unequip_item);
        app.add_observer(handle_item_unequipped);
    }
}

fn handle_unequip_item(
    trigger: Trigger<UnequipItem>,
    mut commands: Commands,
    mut paperdolls: Query<Mut<PaperDoll>>,
    mut items_data: ItemsDataQueryMut,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().item_object_id;
    let skip_db_update = trigger.event().skip_db_update;

    let mut paperdoll = paperdolls.get_mut(character_entity)?;

    // Unequip main item
    let item_id = if let Ok(mut item) = items_data.item_by_object_id_mut(item_object_id) {
        item.unequip();
        paperdoll.unequip(item_object_id);
        commands.trigger_targets(
            ItemUnequipped::new(item_object_id, skip_db_update),
            character_entity,
        );
        item.id()
    } else {
        // Probably destroyed and despawned already
        commands.trigger_targets(
            ItemUnequipped::new(item_object_id, skip_db_update),
            character_entity,
        );
        return Ok(());
    };

    // Check if we need to also unequip ammo from left hand
    let ammo_to_unequip = items_data
        .item_info(item_id)
        .ok()
        .filter(|template| template.kind().bow_or_crossbow())
        .and_then(|weapon_info| {
            paperdoll[DollSlot::LeftHand].and_then(|oid| {
                items_data
                    .info_by_object_id(oid)
                    .ok()
                    .filter(|ammo_info| {
                        ammo_info.kind().ammo() && weapon_info.ammo_matches(ammo_info)
                    })
                    .map(|_| oid)
            })
        });

    if let Some(ammo_oid) = ammo_to_unequip
        && let Ok(mut ammo) = items_data.item_by_object_id_mut(ammo_oid)
    {
        ammo.unequip();
        paperdoll.unequip(ammo_oid);
        commands.trigger_targets(
            ItemUnequipped::new(ammo_oid, skip_db_update),
            character_entity,
        );
    }
    Ok(())
}

fn handle_item_unequipped(
    trigger: Trigger<ItemUnequipped>,
    mut commands: Commands,
    items_data: ItemsDataQuery,
    repo_manager: Res<RepositoryManager>,
    mut attack_effects: Query<Mut<AttackEffects>>,
    mut stats_modifiers: Query<Mut<StatModifiers>>,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().item_object_id;
    let skip_db_update = trigger.event().skip_db_update;
    let item_info = items_data.info_by_object_id(item_object_id)?;

    // Remove stat modifiers
    if let Ok(mut stat_modifiers) = stats_modifiers.get_mut(character_entity)
        && let Some(modifiers) = item_info.stats_modifiers()
    {
        stat_modifiers.unmerge(&modifiers);
    }

    // Reset weapon if this was a weapon
    if let Ok(mut attack_effects) = attack_effects.get_mut(character_entity)
        && item_info.kind().weapon()
    {
        attack_effects.set_weapon(Weapon::default());
    }

    let item = items_data.item_by_object_id(item_object_id)?;
    let unique_item = UniqueItem::new(item_object_id, *item);

    let inventory_update = InventoryUpdate::new(smallvec![unique_item], UpdateType::Modify);
    commands.trigger_targets(GameServerPacket::from(inventory_update), character_entity);

    // Update database
    if !skip_db_update && !repo_manager.is_mock() {
        let items_repository = repo_manager.typed::<ObjectId, items::model::Entity>()?;
        let item_model = items::model::Model::from(unique_item);
        commands.spawn_task(move || async move {
            let mut item_active = item_model.into_active_model();
            item_active.location = ActiveValue::Set(ItemLocationVariant::Inventory);
            items_repository.update(&item_active).await?;

            Ok(())
        });
    }
    commands.trigger_targets(SendUserInfo, character_entity);
    commands.trigger_targets(SendCharInfo, character_entity);

    send_unequipped_system_message(commands.reborrow(), character_entity, unique_item.item());
    Ok(())
}

fn send_unequipped_system_message(mut commands: Commands, character_entity: Entity, item: &Item) {
    if item.enchant_level() > 0 {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::TheEquipmentS1S2HasBeenRemoved,
                vec![
                    SmParam::Number(item.enchant_level().into()),
                    SmParam::Item(item.id().into()),
                ],
            )),
            character_entity,
        );
    } else {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::S1HasBeenDisarmed,
                vec![SmParam::Item(item.id().into())],
            )),
            character_entity,
        );
    }
}
