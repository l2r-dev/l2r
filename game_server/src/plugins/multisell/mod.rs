use bevy::{log, prelude::*};
use bevy_slinet::server::PacketReceiveEvent;
use game_core::{
    account::Account,
    items::{ItemLocation, SpawnNew},
    multisell::{MultisellComponentsPlugin, admin_shop::AdminShopMultiSells},
    network::{
        config::GameServerNetworkConfig, packets::client::GameClientPacket,
        session::PacketReceiveParams,
    },
};

pub struct MultisellPlugin;
impl Plugin for MultisellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MultisellComponentsPlugin);
        app.add_observer(handle_packet);
    }
}

fn handle_packet(
    receive: Trigger<PacketReceiveEvent<GameServerNetworkConfig>>,
    receive_params: PacketReceiveParams,
    accounts: Query<Ref<Account>>,
    admin_shop_items: Res<AdminShopMultiSells>,
    mut items_spawn: EventWriter<SpawnNew>,
) -> Result<()> {
    let event = receive.event();
    let GameClientPacket::MultisellChoose(ref packet) = event.packet else {
        return Ok(());
    };
    let character_entity = receive_params.character(&event.connection.id())?;
    let list_id = packet.list_id();
    // if id >= 1_000_000_000 it is admin shop
    if list_id >= 1_000_000_000.into() {
        let session = receive_params.session(&event.connection.id())?;
        let account = accounts.get(session)?;
        if !account.access().gm() {
            log::error!("Account {:?} is not GM", account.name());
            return Ok(());
        }

        if let Some(entries) = admin_shop_items.get(&list_id) {
            let entry_id = packet.entry_id() as usize;

            if let Some(entry) = entries.get(entry_id) {
                for reward in &entry.rewards {
                    let item_ids = vec![reward.item.id()];
                    let total_count = reward.item.count() * packet.amount();

                    log::debug!(
                        "Admin shop multisell: spawning {} x{} for character {:?}",
                        reward.item.id(),
                        total_count,
                        character_entity
                    );

                    items_spawn.write(SpawnNew {
                        item_ids,
                        count: total_count,
                        item_location: ItemLocation::Inventory,
                        dropped_entity: None,
                        owner: Some(character_entity),
                        silent: false, // Show system messages for multisell purchases
                    });
                }
            } else {
                log::warn!(
                    "MultisellChoose: entry_id {} not found in admin shop list_id {:?}",
                    packet.entry_id(),
                    list_id
                );
            }
        } else {
            log::warn!(
                "MultisellChoose: list_id {:?} not found in admin shop",
                list_id
            );
        }
    } else {
        // MultiSell From assets
        log::debug!(
            "MultisellChoose packet received: list_id={:?}, entry_id={:?}, amount={}",
            packet.list_id(),
            packet.entry_id(),
            packet.amount(),
        );
    }

    Ok(())
}
