use bevy::prelude::*;
use bevy_asset::{
    AssetMetaCheck, UnapprovedPathMode,
    io::{
        AssetReader, AssetReaderError, AssetSource, AssetSourceId, ErasedAssetReader, PathStream,
        Reader,
    },
};
use std::{path::Path, time::Duration};

pub mod binary;
pub mod html;
pub mod text;

pub const ASSET_DIR: &str = "data";
const ASSET_DEBOUNCE_DURATION: f32 = 1.0;

struct L2rAssetReader(Box<dyn ErasedAssetReader>);

impl AssetReader for L2rAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.0.read(path).await
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.0.read_meta(path).await
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        self.0.read_directory(path).await
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        self.0.is_directory(path).await
    }
}

pub struct CustomAssetPlugin;
impl CustomAssetPlugin {
    pub fn custom() -> AssetPlugin {
        AssetPlugin {
            mode: AssetMode::Unprocessed,
            file_path: ASSET_DIR.to_string(),
            processed_file_path: format!("{ASSET_DIR}processed"),
            watch_for_changes_override: Some(true),
            meta_check: AssetMetaCheck::Never,
            unapproved_path_mode: UnapprovedPathMode::Allow,
        }
    }
}

pub struct CustomAssetWatcherPlugin;

impl Plugin for CustomAssetWatcherPlugin {
    fn build(&self, app: &mut App) {
        let debounce_duration = Duration::from_secs_f32(ASSET_DEBOUNCE_DURATION);
        let assets_watcher =
            AssetSource::get_default_watcher(ASSET_DIR.to_string(), debounce_duration);

        app.register_asset_source(
            AssetSourceId::Default,
            AssetSource::build()
                .with_watcher(assets_watcher)
                .with_reader(|| {
                    Box::new(L2rAssetReader(AssetSource::get_default_reader(
                        ASSET_DIR.to_string(),
                    )()))
                }),
        );
    }
}
