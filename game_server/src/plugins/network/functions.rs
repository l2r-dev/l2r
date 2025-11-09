use bevy::prelude::*;
use bevy_ecs::entity::EntityHashSet;
use game_core::{
    character::{self, Character},
    items::ItemsQuery,
    network::{
        broadcast::{BroadcastScope, ServerPacketBroadcast},
        packets::{
            GameServerPacketMetric,
            server::{
                BroadcastCharInfo, CharInfo, CharSelectionInfo, GameServerPacket,
                GameServerPackets, SendCharSelectionInfo, SendUserInfo, UserInfo, UserInfoUpdated,
            },
        },
        session::GameServerSession,
    },
    stats::StatsTableQuery,
};
use l2r_core::{
    metrics::Metrics, model::session::L2rSession, plugins::custom_hierarchy::DespawnChildOf,
};

pub fn send_char_info_handler(
    trigger: Trigger<BroadcastCharInfo>,
    mut commands: Commands,
    characters: Query<character::Query>,
    stats_table_query: StatsTableQuery,
    items_query: ItemsQuery,
) {
    let entity = trigger.target();
    if let Ok(character) = characters.get(entity) {
        let class_tree = stats_table_query.class_tree();
        let race_stats = stats_table_query.race_stats();

        let base_class = class_tree.get_base_class(character.sub_class.class_id());
        let base_class_stats = race_stats.get(*character.race, base_class);

        let char_info = CharInfo::new(
            &character,
            &items_query,
            base_class_stats.base_speed.clone(),
        );

        commands.trigger_targets(
            ServerPacketBroadcast {
                packet: char_info.into(),
                scope: BroadcastScope::Known,
            },
            entity,
        );
    }
}

pub fn send_user_info_handler(
    trigger: Trigger<SendUserInfo>,
    mut commands: Commands,
    characters: Query<character::Query>,
    stats_table_query: StatsTableQuery,
    items_query: ItemsQuery,
) {
    let entity = trigger.target();
    if let Ok(character) = characters.get(entity) {
        let class_tree = stats_table_query.class_tree();
        let race_stats = stats_table_query.race_stats();

        let base_class = class_tree.get_base_class(character.sub_class.class_id());
        let base_class_stats = race_stats.get(*character.race, base_class);

        let user_info = UserInfo::from_query(
            &character,
            &items_query,
            base_class_stats.base_speed.clone(),
        )
        .with_extra();

        commands.trigger_targets(user_info, entity);
    }
}

pub fn send_user_info_when_updated(
    mut event_reader: EventReader<UserInfoUpdated>,
    mut commands: Commands,
    characters: Query<character::Query>,
    stats_table_query: StatsTableQuery,
    items_query: ItemsQuery,
) {
    // We have many systems that can send this events during 1 frame
    // But actually need to send only 1 packet if so.
    let mut affected_entities = EntityHashSet::new();
    for event in event_reader.read() {
        affected_entities.insert(event.0);
    }

    for entity in affected_entities {
        if let Ok(character) = characters.get(entity) {
            let class_tree = stats_table_query.class_tree();
            let race_stats = stats_table_query.race_stats();

            let base_class = class_tree.get_base_class(character.sub_class.class_id());
            let base_class_stats = race_stats.get(*character.race, base_class);

            let user_info = UserInfo::from_query(
                &character,
                &items_query,
                base_class_stats.base_speed.clone(),
            )
            .with_extra();

            commands.trigger_targets(user_info, entity);
        }
    }
}

pub fn send_char_selection_info(
    send: Trigger<SendCharSelectionInfo>,
    mut commands: Commands,
    tables: Query<Ref<character::Table>>,
) {
    let entity = send.target();
    if let Ok(chars_table) = tables.get(entity) {
        commands.trigger_targets(
            GameServerPacket::from(CharSelectionInfo::new(&chars_table)),
            entity,
        );
    }
}

pub fn send_server_packet(
    packet: Trigger<GameServerPacket>,
    sessions: Query<Ref<GameServerSession>>,
    characters: Query<Ref<DespawnChildOf>, With<Character>>,
    metrics: Res<Metrics>,
) -> Result<()> {
    let entity = packet.target();
    let packet = packet.event();

    if let Ok(child_of) = characters.get(entity)
        && let Ok(session) = sessions.get(**child_of)
    {
        metrics.counter(GameServerPacketMetric::PacketsSent)?.inc();
        session.send(packet.clone());
    }

    if let Ok(session) = sessions.get(entity) {
        metrics.counter(GameServerPacketMetric::PacketsSent)?.inc();
        session.send(packet.clone());
    }
    Ok(())
}

pub fn send_multiple_server_packets(packets: Trigger<GameServerPackets>, mut commands: Commands) {
    let entity = packets.target();
    packets.event().iter().for_each(|packet| {
        commands.trigger_targets(packet.clone(), entity);
    });
}
