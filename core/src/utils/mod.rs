use bevy::{log::trace, platform::collections::HashMap, reflect::Reflect};
use bevy_ecs::world::World;
use bevy_mod_scripting::{
    bindings::{AppReflectAllocator, InteropError, ReflectReference},
    prelude::ScriptValue,
};
use log::{Level, log_enabled};
use sea_orm::JsonValue;
use serde::{Deserialize, Deserializer};
use std::{borrow::Cow, env, path::PathBuf};

mod pagination;

pub use pagination::*;

#[inline]
pub fn format_bytes_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[macro_export]
macro_rules! extend_bytes {
    ($bytes:expr, $($value:expr),*) => {
        $(
            $bytes.extend($value.to_le_bytes());
        )*
    };
}

pub fn deserialize_array<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default + Copy,
{
    let v = Vec::<T>::deserialize(deserializer)?;
    if v.len() > N {
        return Err(serde::de::Error::invalid_length(
            v.len(),
            &format!("at most {} elements", N).as_str(),
        ));
    }

    let mut arr = [T::default(); N];
    for (i, val) in v.into_iter().enumerate() {
        if i < N {
            arr[i] = val;
        }
    }
    Ok(arr)
}

pub fn deserialize_array_10<'de, D, T>(deserializer: D) -> Result<[T; 10], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default + Copy,
{
    deserialize_array::<D, T, 10>(deserializer)
}

pub fn deserialize_array_20<'de, D, T>(deserializer: D) -> Result<[T; 20], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default + Copy,
{
    deserialize_array::<D, T, 20>(deserializer)
}

pub fn log_trace_byte_table(data: &[u8], message: &str) {
    if log_enabled!(Level::Trace) {
        let data_size = format!("({})", data.len());
        let header = "offset 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F";

        // Generate a dynamic separator that includes the message and data size
        let combined_message = format!("{data_size} - {message}");
        let dynamic_separator = create_dynamic_separator(&combined_message);

        trace!("{}", dynamic_separator);
        trace!("{}", header);

        for (i, chunk) in data.chunks(16).enumerate() {
            let mut line = format!("{:05X}  ", i << 4);
            for byte in chunk {
                line.push_str(&format!("{byte:02X} "));
            }
            trace!("{}", line);
        }
    }
}

// Helper function to create a dynamic separator that includes the message
fn create_dynamic_separator(combined_message: &str) -> String {
    let base_length = 60; // Minimum total length of the separator line
    let message_length = combined_message.len();
    let total_length = base_length.max(message_length + 2); // Ensure some padding
    let padding_length = total_length - message_length - 10;
    let pad_each_side = padding_length / 2;

    format!(
        "{} [{}] {}",
        "=".repeat(pad_each_side),
        combined_message,
        "=".repeat(pad_each_side)
    )
}

pub fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(manifest_dir)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
            .unwrap()
    }
}

// needed to allow construct with nil values in lua
// https://github.com/bevyengine/bevy/issues/18018
#[macro_export]
macro_rules! register_optional_types {
    ($app:expr, $( $type:ty ),* ) => {
        $(
            $app.register_type::<Option<$type>>()
                .register_type_data::<Option<$type>, ReflectDefault>();
        )*
    };
}

/// Reduces boilerplate code when working with async operations in sync contexts.
pub fn block_on<F, Fut, T, E>(task_fn: F) -> Result<T, InteropError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
    T: Send + 'static,
{
    let task_pool = bevy::tasks::AsyncComputeTaskPool::get();
    let task = task_pool.spawn(task_fn());

    match bevy::tasks::futures_lite::future::block_on(task) {
        Ok(result) => Ok(result),
        Err(e) => Err(InteropError::external(Box::new(e))),
    }
}

pub trait AllocatedReflectExt {
    fn new_allocated<T>(&mut self, model: Option<T>) -> ScriptValue
    where
        T: Reflect + 'static;
}

impl AllocatedReflectExt for World {
    fn new_allocated<T>(&mut self, model: Option<T>) -> ScriptValue
    where
        T: Reflect + 'static,
    {
        if let Some(model) = model
            && let Some(allocator) = self.get_resource_mut::<AppReflectAllocator>()
        {
            return ReflectReference::new_allocated(model, &mut allocator.write()).into();
        }
        ScriptValue::Unit
    }
}

pub trait ScriptValueFromJson {
    fn from_json(value: JsonValue) -> ScriptValue;
}

impl ScriptValueFromJson for ScriptValue {
    fn from_json(value: JsonValue) -> ScriptValue {
        match value {
            JsonValue::Null => ScriptValue::Unit,
            JsonValue::Bool(b) => ScriptValue::Bool(b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ScriptValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ScriptValue::Float(f)
                } else {
                    ScriptValue::String(Cow::Owned(n.to_string()))
                }
            }
            JsonValue::String(s) => ScriptValue::String(Cow::Owned(s)),
            JsonValue::Array(arr) => {
                let list: Vec<ScriptValue> = arr.into_iter().map(ScriptValue::from_json).collect();
                ScriptValue::List(list)
            }
            JsonValue::Object(obj) => {
                let map: HashMap<String, ScriptValue> = obj
                    .into_iter()
                    .map(|(k, v)| (k, ScriptValue::from_json(v)))
                    .collect();
                ScriptValue::Map(map)
            }
        }
    }
}
