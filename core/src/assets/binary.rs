use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, AssetLoader, LoadContext, io::Reader},
};
use std::marker::PhantomData;
use thiserror::Error;

/// Plugin to load your asset type `A` from binary files.
pub struct BinaryAssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

const BINARY_ASSET_BUFFER_SIZE: usize = 4 * 1024 * 1024; // 4 MB buffer size

impl<A> Plugin for BinaryAssetPlugin<A>
where
    A: BinaryAsset + Asset,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .register_asset_loader(BinaryAssetLoader::<A> {
                extensions: self.extensions.clone(),
                _marker: PhantomData,
            });
    }
}

impl<A> BinaryAssetPlugin<A>
where
    A: BinaryAsset + Asset,
{
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}

/// Trait that defines how to load a binary asset.
pub trait BinaryAsset: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryLoaderError>;
}

struct BinaryAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

/// Possible errors that can be produced by [`BinaryAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BinaryLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A custom error for parsing binary data
    #[error("Could not parse the binary data: {0}")]
    BinaryParseError(String),
}

impl<A> AssetLoader for BinaryAssetLoader<A>
where
    A: BinaryAsset + Asset,
{
    type Asset = A;
    type Settings = ();
    type Error = BinaryLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::with_capacity(BINARY_ASSET_BUFFER_SIZE);
        reader.read_to_end(&mut bytes).await?;
        let asset = A::from_bytes(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
