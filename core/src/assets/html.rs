use super::text::{TextAsset, TextAssetPlugin, TextLoaderError};
use bevy::{log, prelude::*};
use cached::proc_macro::cached;
use std::fmt;
use tera::{Context, Tera};

pub struct HtmlPlugin;
impl Plugin for HtmlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TextAssetPlugin::<HtmlAsset>::new(&["html"]));
    }
}

#[derive(Asset, Clone, Default, Reflect)]
pub struct HtmlAsset(String);

impl HtmlAsset {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HtmlAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TextAsset for HtmlAsset {
    fn from_string(text: String) -> Result<Self, TextLoaderError> {
        Ok(HtmlAsset(text))
    }
}

pub trait TeraCachedRender {
    fn render_cached(&self, template_name: &str, context: &Context) -> Result<String, tera::Error>;
}

impl TeraCachedRender for Tera {
    fn render_cached(&self, template_name: &str, context: &Context) -> Result<String, tera::Error> {
        #[cached(
            name = "RENDER_CACHED",
            key = "String",
            convert = r#"{ format!("{}-{:?}", template_name, context) }"#,
            time = 60
        )]
        fn inner_render(tera: &Tera, template_name: String, context: &Context) -> Option<String> {
            tera.render(&template_name, context).ok()
        }

        if let Some(cached_result) = inner_render(self, template_name.to_string(), context) {
            return Ok(cached_result);
        }

        self.render(template_name, context)
    }
}

pub trait TeraHtmlTemplater {
    const DEFAULT_TEMPLATE_PATH: &'static str = "_common/default.html";

    fn templater(&self) -> &Tera;
    fn templater_mut(&mut self) -> &mut Tera;

    /// Renders the default HTML template with the provided context.
    fn render_default(&self, context: &Context) -> Result<String, tera::Error> {
        self.render(Self::DEFAULT_TEMPLATE_PATH, context)
    }

    /// Renders a template with automatic fallback to the default template on error.
    fn render_with_fallback(
        &self,
        template_path: &str,
        context: &Context,
    ) -> Result<String, tera::Error> {
        match self.render(template_path, context) {
            Ok(result) => Ok(result),
            Err(error) => {
                log::warn!(
                    "Failed to render template at path '{}', falling back to default: {}",
                    template_path,
                    error
                );
                self.render_default(context)
            }
        }
    }

    /// Reload all templates from disk
    fn reload(&mut self) {
        if let Err(error) = self.templater_mut().full_reload() {
            log::error!("Failed to reload templates: {}", error);
        }
    }

    fn render(&self, template_path: &str, context: &Context) -> Result<String, tera::Error> {
        // self.templater().render_cached(template_path, context)
        self.templater().render(template_path, context)
    }
}

pub trait TeraContext {
    fn tera_context(&self) -> Context;
}
