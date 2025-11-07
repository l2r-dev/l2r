use bevy::prelude::*;
use bevy_defer::AsyncCommandsExtension;
use game_core::{
    items::{
        self, DollSlot, ItemLocationVariant, ItemUnequipped, ItemUnequippedMessage,
        ItemsDataAccess, ItemsDataQuery, ItemsDataQueryMut, PaperDoll, UnequipItem, UniqueItem,
        UpdateType,
    },
    network::packets::server::{
        BroadcastCharInfo, GameServerPacket, InventoryUpdate, SendUserInfo, SystemMessage,
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
        app.add_observer(handle_unequip_item)
            .add_observer(handle_item_unequipped)
            .add_observer(send_unequipped_system_message);
    }
}

pub fn process_unequip(
    entity: Entity,
    item_object_id: ObjectId,
    mut commands: Commands,
    items_data: &mut ItemsDataQueryMut,
    mut paperdolls: Query<Mut<PaperDoll>>,
    skip_db_update: bool,
    repo_manager: &RepositoryManager,
) -> Result<()> {
    let mut paperdoll = paperdolls.get_mut(entity)?;

    // Unequip main item
    let item_id = if let Ok(mut item) = items_data.item_by_object_id_mut(item_object_id) {
        item.unequip();
        paperdoll.unequip(item_object_id);
        commands.trigger_targets(ItemUnequipped(item_object_id), entity);
        item.id()
    } else {
        // Probably destroyed and despawned already
        commands.trigger_targets(ItemUnequipped(item_object_id), entity);
        return Ok(());
    };

    debug!("Unequipped item {:?} from entity {:?}", item_id, entity);

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

    debug!(
        "Also unequipping ammo {:?} from entity {:?}",
        ammo_to_unequip, entity
    );

    if let Some(item_object_id) = ammo_to_unequip {
        commands.trigger_targets(
            UnequipItem {
                item_object_id,
                skip_db_update: false,
            },
            entity,
        );
    }

    let item = items_data.item_by_object_id(item_object_id)?;
    let unique_item = UniqueItem::new(item_object_id, *item);

    let inventory_update = InventoryUpdate::new(smallvec![unique_item], UpdateType::Modify);
    commands.trigger_targets(GameServerPacket::from(inventory_update), entity);

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

    Ok(())
}

fn handle_unequip_item(
    trigger: Trigger<UnequipItem>,
    mut commands: Commands,
    mut paperdolls: Query<Mut<PaperDoll>>,
    mut items_data: ItemsDataQueryMut,
    repo_manager: Res<RepositoryManager>,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().item_object_id;
    let skip_db_update = trigger.event().skip_db_update;

    process_unequip(
        character_entity,
        item_object_id,
        commands.reborrow(),
        &mut items_data,
        paperdolls.reborrow(),
        skip_db_update,
        repo_manager.as_ref(),
    )
}

fn handle_item_unequipped(
    trigger: Trigger<ItemUnequipped>,
    mut commands: Commands,
    items_data: ItemsDataQuery,
    mut attack_effects: Query<Mut<AttackEffects>>,
    mut stats_modifiers: Query<Mut<StatModifiers>>,
) -> Result<()> {
    let character_entity = trigger.target();
    let item_object_id = trigger.event().0;
    let item_info = items_data.info_by_object_id(item_object_id)?;

    // Stat calculations
    if let Ok(mut stat_modifiers) = stats_modifiers.get_mut(character_entity)
        && let Some(modifiers) = item_info.stats_modifiers()
    {
        stat_modifiers.unmerge(&modifiers);
    }

    if let Ok(mut attack_effects) = attack_effects.get_mut(character_entity)
        && item_info.kind().weapon()
    {
        attack_effects.set_weapon(Weapon::default());
    }

    let item = items_data.item_by_object_id(item_object_id)?;
    commands.trigger_targets(SendUserInfo, character_entity);
    commands.trigger_targets(BroadcastCharInfo, character_entity);
    commands.trigger_targets(ItemUnequippedMessage(*item), character_entity);

    Ok(())
}

pub fn send_unequipped_system_message(
    trigger: Trigger<ItemUnequippedMessage>,
    mut commands: Commands,
) {
    let item = trigger.event().0;
    let entity = trigger.target();
    if item.enchant_level() > 0 {
        commands.trigger_targets(
            GameServerPacket::from(SystemMessage::new(
                SmId::TheEquipmentS1S2HasBeenRemoved,
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
                SmId::S1HasBeenDisarmed,
                vec![SmParam::Item(item.id().into())],
            )),
            entity,
        );
    }
}
