use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub fn deserialize_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    let value = Value::deserialize(deserializer)?;
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .ok_or_else(|| serde::de::Error::custom("invalid unsigned ID"))
}

pub(super) fn deserialize_counter<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i64, D::Error> {
    let value = Value::deserialize(deserializer)?;
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .ok_or_else(|| serde::de::Error::custom("invalid counter"))
}

pub(super) fn deserialize_amount<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<f64, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let amount = match &value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    };
    amount
        .filter(|amount| amount.is_finite())
        .ok_or_else(|| serde::de::Error::custom("invalid amount"))
}
