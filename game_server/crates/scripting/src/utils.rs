//! ScriptValue argument parsing utilities
//!
//! This module provides a trait-based approach for parsing ScriptValue arguments
//! into structured data types, replacing individual utility functions with a more
//! extensible and type-safe system.

use crate::prelude::ScriptValue;
use bevy::{ecs::entity::Entity, math::Vec3};
use bevy_mod_scripting::bindings::{
    FunctionCallContext, InteropError, ReflectReference, ScriptComponentRegistration, WorldGuard,
};
use std::any::TypeId;

/// Trait for parsing ScriptValue into structured argument types
///
/// This trait provides a type-safe way to extract arguments from ScriptValue,
/// enabling consistent error handling and extensible argument parsing patterns.
pub trait ScriptValueToArguments<'a>: Sized {
    /// Parse the ScriptValue into the implementing type
    ///
    /// # Arguments
    /// * `data` - The ScriptValue to parse
    ///
    /// # Returns
    /// * `Ok(Self)` - Successfully parsed arguments
    /// * `Err(InteropError)` - Parsing failed with detailed error information
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError>;
}

pub trait ScriptValueToArgumentsDowncast<'a>: Sized {
    fn from_script_value_downcast(
        data: &'a ScriptValue,
        ctx: FunctionCallContext,
    ) -> Result<Self, InteropError>;
}

/// Generic string with any ScriptValue pair
/// Expects input: [string_value, any_script_value]
pub struct StringWithValue<'a> {
    pub string: String,
    pub value: &'a ScriptValue,
}

impl<'a> ScriptValueToArguments<'a> for StringWithValue<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 2 => {
                let string_value = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let value = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                let string = match string_value {
                    ScriptValue::String(s) => s.to_string(),
                    _ => {
                        return Err(InteropError::string_type_mismatch(
                            "String".to_string(),
                            None,
                        ));
                    }
                };

                Ok(StringWithValue { string, value })
            }
            _ => Err(InteropError::argument_count_mismatch(2, 1)),
        }
    }
}

/// Generic string with optional ScriptValue
/// Expects input: string_value or [string_value, optional_value]
pub struct StringWithOptional<'a> {
    pub string: String,
    pub optional_value: Option<&'a ScriptValue>,
}

impl<'a> ScriptValueToArguments<'a> for StringWithOptional<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::String(s) => {
                // Single string format
                Ok(StringWithOptional {
                    string: s.to_string(),
                    optional_value: None,
                })
            }
            ScriptValue::List(list) if list.len() == 2 => {
                // [string, optional_value] format
                let string_value = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let optional_value = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                let string = match string_value {
                    ScriptValue::String(s) => s.to_string(),
                    _ => {
                        return Err(InteropError::string_type_mismatch(
                            "String".to_string(),
                            None,
                        ));
                    }
                };

                Ok(StringWithOptional {
                    string,
                    optional_value: Some(optional_value),
                })
            }
            _ => Err(InteropError::value_mismatch(
                TypeId::of::<String>(),
                data.clone(),
            )),
        }
    }
}

/// Single reference argument
/// Expects input: reference_value
pub struct SingleReference<'a> {
    pub reference: &'a ReflectReference,
}

impl<'a> ScriptValueToArguments<'a> for SingleReference<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::Reference(reference) => Ok(SingleReference { reference }),
            _ => Err(InteropError::value_mismatch(
                TypeId::of::<ReflectReference>(),
                data.clone(),
            )),
        }
    }
}

/// Two reference arguments
/// Expects input: [ref1, ref2]
pub struct TwoReferences<'a> {
    pub first: &'a ReflectReference,
    pub second: &'a ReflectReference,
}

impl<'a> ScriptValueToArguments<'a> for TwoReferences<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 2 => {
                let first = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let second = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                match (first, second) {
                    (ScriptValue::Reference(ref1), ScriptValue::Reference(ref2)) => {
                        Ok(TwoReferences {
                            first: ref1,
                            second: ref2,
                        })
                    }
                    _ => Err(InteropError::type_mismatch(
                        TypeId::of::<ReflectReference>(),
                        None,
                    )),
                }
            }
            _ => Err(InteropError::argument_count_mismatch(2, 1)),
        }
    }
}

/// Three reference arguments
/// Expects input: [ref1, ref2, ref3]
pub struct ThreeReferences<'a> {
    pub first: &'a ReflectReference,
    pub second: &'a ReflectReference,
    pub third: &'a ReflectReference,
}

impl<'a> ScriptValueToArguments<'a> for ThreeReferences<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 3 => {
                let first = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(3, 1))?;
                let second = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(3, 2))?;
                let third = list
                    .get(2)
                    .ok_or_else(|| InteropError::argument_count_mismatch(3, 2))?;

                match (first, second, third) {
                    (
                        ScriptValue::Reference(ref1),
                        ScriptValue::Reference(ref2),
                        ScriptValue::Reference(ref3),
                    ) => Ok(ThreeReferences {
                        first: ref1,
                        second: ref2,
                        third: ref3,
                    }),
                    _ => Err(InteropError::type_mismatch(
                        TypeId::of::<ReflectReference>(),
                        None,
                    )),
                }
            }
            _ => Err(InteropError::argument_count_mismatch(3, 1)),
        }
    }
}

/// Model reference with optional secondary argument
/// Expects input: model_ref or [model_ref, extra_arg]
pub struct ModelWithOptionalArg<'a> {
    pub model_ref: &'a ReflectReference,
    pub extra_arg: Option<&'a ScriptValue>,
}

impl<'a> ScriptValueToArguments<'a> for ModelWithOptionalArg<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::Reference(model_ref) => {
                // Single model_ref format
                Ok(ModelWithOptionalArg {
                    model_ref,
                    extra_arg: None,
                })
            }
            ScriptValue::List(list) if list.len() == 2 => {
                // [model_ref, extra_arg] format
                let model_value = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let extra_value = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                if let ScriptValue::Reference(model_ref) = model_value {
                    Ok(ModelWithOptionalArg {
                        model_ref,
                        extra_arg: Some(extra_value),
                    })
                } else {
                    Err(InteropError::value_mismatch(
                        TypeId::of::<ReflectReference>(),
                        model_value.clone(),
                    ))
                }
            }
            _ => Err(InteropError::value_mismatch(
                TypeId::of::<ReflectReference>(),
                data.clone(),
            )),
        }
    }
}

/// Exact list with specified length
/// Generic over the expected count for compile-time safety
pub struct ExactList<'a, const N: usize> {
    pub items: &'a [ScriptValue],
}

impl<'a, const N: usize> ScriptValueToArguments<'a> for ExactList<'a, N> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == N => Ok(ExactList {
                items: list.as_slice(),
            }),
            ScriptValue::List(list) => Err(InteropError::argument_count_mismatch(N, list.len())),
            _ => Err(InteropError::argument_count_mismatch(N, 1)),
        }
    }
}

/// String argument with context
/// Expects input: string_value
pub struct StringArg {
    pub value: String,
}

impl<'a> ScriptValueToArguments<'a> for StringArg {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::String(s) => Ok(StringArg {
                value: s.to_string(),
            }),
            _ => Err(InteropError::string_type_mismatch(
                "String".to_string(),
                None,
            )),
        }
    }
}

/// Integer argument
/// Expects input: integer_value
pub struct IntegerArg {
    pub value: i64,
}

impl<'a> ScriptValueToArguments<'a> for IntegerArg {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::Integer(i) => Ok(IntegerArg { value: *i }),
            _ => Err(InteropError::value_mismatch(
                TypeId::of::<i64>(),
                data.clone(),
            )),
        }
    }
}

/// Generic two values of any ScriptValue types
/// Expects input: [first_value, second_value]
pub struct TwoValues<'a> {
    pub first: &'a ScriptValue,
    pub second: &'a ScriptValue,
}

impl<'a> ScriptValueToArguments<'a> for TwoValues<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 2 => {
                let first = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let second = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                Ok(TwoValues { first, second })
            }
            _ => Err(InteropError::argument_count_mismatch(2, 1)),
        }
    }
}

/// Generic three values of any ScriptValue types
/// Expects input: [first_value, second_value, third_value]
pub struct ThreeValues<'a> {
    pub first: &'a ScriptValue,
    pub second: &'a ScriptValue,
    pub third: &'a ScriptValue,
}

impl<'a> ScriptValueToArguments<'a> for ThreeValues<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 3 => {
                let first = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(3, 1))?;
                let second = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(3, 2))?;
                let third = list
                    .get(2)
                    .ok_or_else(|| InteropError::argument_count_mismatch(3, 2))?;

                Ok(ThreeValues {
                    first,
                    second,
                    third,
                })
            }
            _ => Err(InteropError::argument_count_mismatch(3, 1)),
        }
    }
}

/// Generic value with optional second value
/// Expects input: value or [value, optional_value]
pub struct ValueWithOptional<'a> {
    pub value: &'a ScriptValue,
    pub optional_value: Option<&'a ScriptValue>,
}

impl<'a> ScriptValueToArguments<'a> for ValueWithOptional<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 2 => {
                // [value, optional_value] format
                let value = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let optional_value = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                Ok(ValueWithOptional {
                    value,
                    optional_value: Some(optional_value),
                })
            }
            _ => {
                // Single value format
                Ok(ValueWithOptional {
                    value: data,
                    optional_value: None,
                })
            }
        }
    }
}

/// Entity with component registration arguments
/// Expects input: [entity_ref, component_registration_ref]
pub struct EntityWithComponentRegistration {
    pub entity: Entity,
    pub component_registration: ScriptComponentRegistration,
}

impl<'a> ScriptValueToArgumentsDowncast<'a> for EntityWithComponentRegistration {
    fn from_script_value_downcast(
        data: &'a ScriptValue,
        ctx: FunctionCallContext,
    ) -> Result<Self, InteropError> {
        match data {
            ScriptValue::List(list) if list.len() == 2 => {
                let entity_value = list
                    .first()
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
                let component_registration_value = list
                    .get(1)
                    .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

                if let (
                    ScriptValue::Reference(entity_ref),
                    ScriptValue::Reference(component_registration_ref),
                ) = (entity_value, component_registration_value)
                {
                    let entity = entity_ref.downcast::<Entity>(ctx.world()?)?;
                    let component_registration =
                        component_registration_ref
                            .downcast::<ScriptComponentRegistration>(ctx.world()?)?;

                    Ok(EntityWithComponentRegistration {
                        entity,
                        component_registration,
                    })
                } else {
                    Err(InteropError::type_mismatch(
                        std::any::TypeId::of::<Entity>(),
                        None,
                    ))
                }
            }
            _ => Err(InteropError::argument_count_mismatch(2, 1)),
        }
    }
}

/// Helper function to extract Vec3 from ScriptValue
///
/// # Arguments
/// * `value` - The ScriptValue to extract Vec3 from
/// * `world_guard` - WorldGuard for downcasting references
///
/// # Returns
/// * `Ok(Vec3)` - Successfully extracted Vec3
/// * `Err(InteropError)` - Failed to extract Vec3
pub fn extract_vec3(value: &ScriptValue, world_guard: WorldGuard) -> Result<Vec3, InteropError> {
    match value {
        ScriptValue::Reference(ref_val) => {
            let vec3 = ref_val.downcast::<Vec3>(world_guard)?;
            Ok(vec3)
        }
        _ => Err(InteropError::type_mismatch(
            std::any::TypeId::of::<Vec3>(),
            None,
        )),
    }
}

/// Helper function to extract two Vec3 positions from ScriptValue
///
/// Expects a list with two Vec3 values.
///
/// # Arguments
/// * `data` - The ScriptValue containing a list of two Vec3 positions
/// * `world_guard` - WorldGuard for downcasting references
///
/// # Returns
/// * `Ok((Vec3, Vec3))` - Successfully extracted both positions
/// * `Err(InteropError)` - Failed to extract positions
pub fn extract_positions(
    data: &ScriptValue,
    world_guard: WorldGuard,
) -> Result<(Vec3, Vec3), InteropError> {
    match data {
        ScriptValue::List(list) if list.len() == 2 => {
            let first = list
                .first()
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 0))?;
            let second = list
                .get(1)
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;

            let first_vec3 = extract_vec3(first, world_guard.clone())?;
            let second_vec3 = extract_vec3(second, world_guard)?;

            Ok((first_vec3, second_vec3))
        }
        _ => Err(InteropError::invariant(
            "Expected a list with two Vec3 positions".to_string(),
        )),
    }
}
