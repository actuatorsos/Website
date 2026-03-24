//! SurrealDB SDK v3 Compatibility Helpers
//!
//! Provides extension traits to bridge the SDK v2 -> v3 migration.
//! In v3, `.take_json::<T>()` requires `T: SurrealValue` instead of `T: Deserialize`.
//! These helpers go through `serde_json::Value` (which impls `SurrealValue`) and then
//! use serde to deserialize into the target type.

use serde::de::DeserializeOwned;
use surrealdb::IndexedResults;

/// Extension trait for `IndexedResults` (formerly `Response` in SDK v2).
///
/// Provides `.take_json::<T>(idx)` for types that only implement `Deserialize`.
/// T can be `Vec<SomeStruct>`, `Option<SomeStruct>`, or any `Deserialize` type.
pub trait TakeJson {
    /// Take a query result by index, deserializing via `serde_json::Value`.
    /// Works for `Vec<T>`, `Option<T>`, or single `T` results.
    fn take_json<T: DeserializeOwned>(&mut self, index: usize) -> Result<T, surrealdb::Error>;
}

impl TakeJson for IndexedResults {
    fn take_json<T: DeserializeOwned>(&mut self, index: usize) -> Result<T, surrealdb::Error> {
        tracing::debug!("take_json: extracting index {}", index);
        let vals: Vec<serde_json::Value> = self.take(index)?;
        tracing::debug!("take_json: got {} values", vals.len());
        let json_arr = serde_json::Value::Array(vals);
        // Try to deserialize the array directly (works for Vec<T>)
        // If that fails, try the first element (works for Option<T> and single T)
        serde_json::from_value::<T>(json_arr.clone()).or_else(|_| {
            if let serde_json::Value::Array(arr) = json_arr {
                match arr.into_iter().next() {
                    Some(v) => serde_json::from_value(v)
                        .map_err(|e| surrealdb::Error::internal(format!("Deserialization error: {}", e))),
                    None => serde_json::from_value(serde_json::Value::Null)
                        .map_err(|e| surrealdb::Error::internal(format!("Deserialization error (empty result): {}", e))),
                }
            } else {
                Err(surrealdb::Error::internal("Unexpected result type".to_string()))
            }
        })
    }
}

use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

/// Create a record with content, serializing to serde_json::Value to avoid SurrealValue requirement.
/// Returns the created record deserialized to R. Content type C can differ from result type R.
pub async fn create_with_content<R: DeserializeOwned, C: Serialize>(
    db: &Surreal<Client>,
    table: &str,
    content: &C,
) -> Result<Option<R>, surrealdb::Error> {
    let data = serde_json::to_value(content)
        .map_err(|e| surrealdb::Error::internal(format!("Serialization error: {}", e)))?;
    let result: Option<serde_json::Value> = db
        .create::<Option<serde_json::Value>>(table)
        .content(data)
        .await?;
    match result {
        Some(v) => serde_json::from_value(v)
            .map(Some)
            .map_err(|e| surrealdb::Error::internal(format!("Deserialization error: {}", e))),
        None => Ok(None),
    }
}

/// Update a record with merge, returning deserialized result.
pub async fn update_merge<T: DeserializeOwned, I: Into<surrealdb::types::RecordIdKey>>(
    db: &Surreal<Client>,
    resource: (&str, I),
    merge_data: serde_json::Value,
) -> Result<Option<T>, surrealdb::Error> {
    let val: Option<serde_json::Value> = db
        .update(resource)
        .merge(merge_data)
        .await?;
    match val {
        Some(v) => serde_json::from_value(v)
            .map(Some)
            .map_err(|e| surrealdb::Error::internal(format!("Deserialization error: {}", e))),
        None => Ok(None),
    }
}

/// Select a single record by (table, id) and deserialize via serde_json::Value.
pub async fn select_one<T: DeserializeOwned, I: Into<surrealdb::types::RecordIdKey>>(
    db: &Surreal<Client>,
    resource: (&str, I),
) -> Result<Option<T>, surrealdb::Error> {
    let val: Option<serde_json::Value> = db.select(resource).await?;
    match val {
        Some(v) => serde_json::from_value(v)
            .map(Some)
            .map_err(|e| surrealdb::Error::internal(format!("Deserialization error: {}", e))),
        None => Ok(None),
    }
}

/// Select all records from a table and deserialize via serde_json::Value.
pub async fn select_all<T: DeserializeOwned>(
    db: &Surreal<Client>,
    table: &str,
) -> Result<Vec<T>, surrealdb::Error> {
    let vals: Vec<serde_json::Value> = db.select(table).await?;
    vals.into_iter()
        .map(|v| serde_json::from_value(v)
            .map_err(|e| surrealdb::Error::internal(format!("Deserialization error: {}", e))))
        .collect()
}

/// Extract the raw ID string from a SurrealDB record ID `serde_json::Value`.
///
/// SurrealDB record IDs are serialized as `"table:key"`. This function
/// extracts just the `key` part. If the value is not a string or has no colon,
/// falls back to `to_string()`.
pub fn thing_to_raw(val: &serde_json::Value) -> String {
    match val.as_str() {
        Some(s) => {
            if let Some(pos) = s.find(':') {
                s[pos + 1..].to_string()
            } else {
                s.to_string()
            }
        }
        None => val.to_string().trim_matches('"').to_string(),
    }
}

/// Extract the full record ID string (table:key) from a `serde_json::Value`.
pub fn thing_to_string(val: &serde_json::Value) -> String {
    match val.as_str() {
        Some(s) => s.to_string(),
        None => val.to_string().trim_matches('"').to_string(),
    }
}
