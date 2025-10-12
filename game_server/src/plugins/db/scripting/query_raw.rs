use bevy::prelude::*;
use l2r_core::{
    db::DbConnection,
    utils::{ScriptValueFromJson, block_on},
};
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::ScriptValue,
    utils::ScriptValueToArguments,
};
use sea_orm::{ConnectionTrait, FromQueryResult, JsonValue, QueryResult, sea_query::Value};

/// Expects input as Map with keys: "sql", "params" (optional), "return_multiple" (optional)
pub struct QueryArgs<'a> {
    pub sql: String,
    pub params: Vec<Value>,
    pub return_multiple: bool,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScriptValueToArguments<'a> for QueryArgs<'a> {
    fn from_script_value(data: &'a ScriptValue) -> Result<Self, InteropError> {
        match data {
            ScriptValue::Map(map) => {
                let sql = map.get("sql").ok_or_else(|| {
                    InteropError::invariant("Missing 'sql' parameter".to_string())
                })?;

                let sql = match sql {
                    ScriptValue::String(s) => s.to_string(),
                    _ => {
                        return Err(InteropError::invariant(
                            "'sql' must be a string".to_string(),
                        ));
                    }
                };

                let params = map
                    .get("params")
                    .map(parse_params_array)
                    .transpose()?
                    .unwrap_or_default();

                let return_multiple = map
                    .get("return_multiple")
                    .map(|v| match v {
                        ScriptValue::Bool(b) => Ok(*b),
                        _ => Err(InteropError::invariant(
                            "'return_multiple' must be a boolean".to_string(),
                        )),
                    })
                    .transpose()?
                    .unwrap_or(false);

                Ok(QueryArgs {
                    sql,
                    params,
                    return_multiple,
                    _phantom: std::marker::PhantomData,
                })
            }
            _ => Err(InteropError::invariant(
                "Expected Map with query parameters".to_string(),
            )),
        }
    }
}

fn parse_params_array(value: &ScriptValue) -> Result<Vec<Value>, InteropError> {
    match value {
        ScriptValue::List(arr) => {
            let mut params = Vec::with_capacity(arr.len());
            for item in arr {
                let param_value = script_value_to_sea_orm_value(item)?;
                params.push(param_value);
            }
            Ok(params)
        }
        _ => Err(InteropError::invariant(
            "Parameters must be an array".to_string(),
        )),
    }
}

fn script_value_to_sea_orm_value(value: &ScriptValue) -> Result<Value, InteropError> {
    match value {
        ScriptValue::Unit => Ok(Value::String(None)),
        ScriptValue::Bool(b) => Ok(Value::Bool(Some(*b))),
        ScriptValue::Integer(i) => Ok(Value::BigInt(Some(*i))),
        ScriptValue::Float(f) => Ok(Value::Double(Some(*f))),
        ScriptValue::String(s) => Ok(Value::String(Some(Box::new(s.to_string())))),
        // For complex types, convert to JSON string
        _ => {
            let json_str = format!("{:?}", value); // Simple debug representation
            Ok(Value::String(Some(Box::new(json_str))))
        }
    }
}

/// Script binding for executing raw SQL queries that return QueryResult
///
/// Expected parameters (as Map):
/// - repository_name: String - Name of the repository to use
/// - sql: String - Raw SQL query
/// - params: Array - Parameters for the SQL query to replace placeholders (optional)
/// - return_multiple: bool - Whether to return multiple results (optional, default: false)
///
/// Returns: JSON representation of query results
pub(crate) fn script_query_raw(
    ctx: FunctionCallContext,
    data: ScriptValue,
) -> std::result::Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    // Parse arguments from ScriptValue using ScriptValueToArguments
    let args = QueryArgs::from_script_value(&data)?;

    world_guard.with_global_access(|world| {
        let connection = world.resource::<DbConnection>().connection();
        let backend = connection.get_database_backend();
        let statement = sea_orm::Statement::from_sql_and_values(backend, &args.sql, args.params);

        // Execute query based on whether we want single or multiple results
        let json_result =
            if args.return_multiple {
                block_on(|| async move {
                    connection.query_all(statement).await.map_err(|e| {
                        InteropError::invariant(format!("Query execution failed: {}", e))
                    })
                })
                .map(query_results_to_json_array)?
            } else {
                block_on(|| async move {
                    connection.query_one(statement).await.map_err(|e| {
                        InteropError::invariant(format!("Query execution failed: {}", e))
                    })
                })
                .map(query_result_to_json)?
            };

        Ok(ScriptValue::from_json(json_result))
    })?
}

fn query_result_to_json(result: Option<QueryResult>) -> JsonValue {
    match result {
        Some(query_result) => match JsonValue::from_query_result(&query_result, "") {
            Ok(json_value) => json_value,
            Err(_) => JsonValue::Null,
        },
        None => JsonValue::Null,
    }
}

fn query_results_to_json_array(results: Vec<QueryResult>) -> JsonValue {
    let json_results: Vec<JsonValue> = results
        .into_iter()
        .map(|result| query_result_to_json(Some(result)))
        .collect();

    JsonValue::Array(json_results)
}
