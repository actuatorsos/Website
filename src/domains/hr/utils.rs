use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Value>::deserialize(deserializer)?;
    match opt {
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::String(s)) => s.parse::<f64>().map(Some).map_err(serde::de::Error::custom),
        Some(Value::Null) => Ok(None),
        None => Ok(None),
        _ => Err(serde::de::Error::custom(
            "expected a number or numeric string",
        )),
    }
}
