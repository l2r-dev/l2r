use bevy::{log, prelude::*};
use bevy_ecs::system::SystemParam;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    items::{
        CharacterInventories, ConsumableKind, EquipItems, Item, ItemsDataQuery, Kind, UnequipItems,
        UseShot,
    },
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
    object_id::{ObjectIdManager, QueryByObjectId},
};

pub struct UseItemPlugin;
impl Plugin for UseItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle);
    }
}

#[derive(SystemParam)]
struct UseItemParams<'w, 's> {
    receive_params: PacketReceiveParams<'w, 's>,
    character_inventories: CharacterInventories<'w, 's>,
    items: Query<'w, 's, (Entity, Ref<'static, Item>)>,
    items_data_query: ItemsDataQuery<'w>,
    equip_items: EventWriter<'w, EquipItems>,
    unequip_items: EventWriter<'w, UnequipItems>,
    use_shot_events: EventWriter<'w, UseShot>,
    object_id_manager: Res<'w, ObjectIdManager>,
}

//TODO: Нужен UseKind (можно ли использовать, когда есть ActiveAction)
fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    mut params: UseItemParams,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::UseItem(ref packet) = event.packet {
        let character_entity = params.receive_params.character(&event.connection.id())?;
        let inventory = params.character_inventories.get(character_entity)?;
        let item_object_id = inventory.get_item(packet.object_id)?;
        let (item_entity, item) = params
            .items
            .by_object_id(item_object_id, params.object_id_manager.as_ref())?;
        let item_info = params.items_data_query.get_item_info(item.id())?;

        log::debug!(
            "Use item: {} (entity: {}, object_id: {})",
            item_info.name(),
            item_entity,
            item_object_id,
        );

        if let Kind::Consumable(ConsumableKind::Shot(_)) = item_info.kind() {
            params
                .use_shot_events
                .write(UseShot::new(character_entity, item_entity));
            return Ok(());
        }

        if item_info.bodypart().is_some() {
            if item.equipped() {
                params
                    .unequip_items
                    .write(UnequipItems::new(character_entity, vec![item_object_id]));
            } else {
                params
                    .equip_items
                    .write(EquipItems::new(character_entity, vec![item_object_id]));
            }
        } else {
            item_info.use_item();
        }
    }
    Ok(())
}
