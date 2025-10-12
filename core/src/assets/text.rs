use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use std::marker::PhantomData;
use thiserror::Error;

pub struct TextAssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

impl<A> Plugin for TextAssetPlugin<A>
where
    A: TextAsset + Asset,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .register_asset_loader(TextAssetLoader::<A> {
                extensions: self.extensions.clone(),
                _marker: PhantomData,
            });
    }
}

impl<A> TextAssetPlugin<A>
where
    A: TextAsset + Asset,
{
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}

pub trait TextAsset: Sized {
    fn from_string(text: String) -> Result<Self, TextLoaderError>;
}

struct TextAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TextLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse the text data: {0}")]
    TextParseError(String),
}

impl<A> AssetLoader for TextAssetLoader<A>
where
    A: TextAsset + Asset,
{
    type Asset = A;
    type Settings = ();
    type Error = TextLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = String::from_utf8(bytes).map_err(|e| {
            TextLoaderError::TextParseError(format!("Failed to parse text asset from bytes: {e}"))
        })?;
        let asset = A::from_string(text)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
