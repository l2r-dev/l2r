use bevy::{log, prelude::*};
use bevy_ecs::system::SystemParam;
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    items::{
        ConsumableKind, EquipItem, InventoriesQuery, ItemsDataAccess, ItemsDataQuery, Kind,
        UnequipItem, UseShot,
    },
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
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
    inventories: InventoriesQuery<'w, 's>,
    items_data: ItemsDataQuery<'w, 's>,
    use_shot_events: EventWriter<'w, UseShot>,
}

//TODO: Нужен UseKind (можно ли использовать, когда есть ActiveAction)
fn handle(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    mut params: UseItemParams,
    mut commands: Commands,
) -> Result<()> {
    let event = receive.event();
    if let GameClientPacket::UseItem(ref packet) = event.packet {
        let character_entity = params.receive_params.character(&event.connection.id())?;
        let inventory = params.inventories.get(character_entity)?;
        let item_object_id = inventory.get_item(packet.object_id)?;
        let item_entity = params.items_data.entity(item_object_id)?;

        let item = params.items_data.item_by_object_id(item_object_id)?;
        let item_info = params.items_data.item_info(item.id())?;

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
                commands.trigger_targets(
                    UnequipItem {
                        item_object_id,
                        skip_db_update: false,
                    },
                    character_entity,
                );
            } else {
                commands.trigger_targets(EquipItem(item_object_id), character_entity);
            }
        } else {
            item_info.use_item();
        }
    }
    Ok(())
}
