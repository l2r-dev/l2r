use bevy::{log, prelude::*};
use game_core::teleport::*;
use l2r_core::chronicles::CHRONICLE;
use std::path::PathBuf;

pub struct TeleportDestinationsPlugin;
impl Plugin for TeleportDestinationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TeleportDestinationsComponentsPlugin,));

        app.init_resource::<TeleportDestinationsHandle>();

        app.add_systems(Startup, load_assets)
            .add_systems(Update, update_assets);
    }
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut teleport_destinations_handle: ResMut<TeleportDestinationsHandle>,
) {
    let mut path = PathBuf::from("teleport");
    path.push(CHRONICLE);
    path.push("destinations");
    path.set_extension("json");

    let handle: Handle<TeleportDestinations> = asset_server.load(path.clone());
    **teleport_destinations_handle = handle;
}

fn update_assets(
    handle: Res<TeleportDestinationsHandle>,
    mut events: EventReader<AssetEvent<TeleportDestinations>>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && handle.id() == *id
        {
            log::info!("Teleport destinations updated");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2r_core::{assets::ASSET_DIR, chronicles::CHRONICLE, utils::get_base_path};
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test_parse_all_from_json() {
        let mut path = get_base_path();
        path.push(ASSET_DIR);
        path.push("teleport");
        path.push(CHRONICLE);
        path.push("destinations");
        path.set_extension("json");

        let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open file: {:?}", path));
        let reader = BufReader::new(file);

        let result: TeleportDestinations = from_reader(reader)
            .unwrap_or_else(|_| panic!("Failed to parse NPC from JSON: {:?}", path));

        assert!(!result.is_empty());
    }
}
