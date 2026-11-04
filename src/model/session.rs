use super::PushProtocol;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

// Intentionally no Debug: live keys and protocol fields contain secrets.
#[derive(Clone)]
pub struct LiveSession {
    pub live_key: String,
    pub protocols: Vec<PushProtocol>,
}

impl<'de> Deserialize<'de> for LiveSession {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = Value::deserialize(deserializer)?;
        let live_key = data
            .get("live_key")
            .map(|key| match key {
                Value::String(value) => value.clone(),
                Value::Number(value) if value.is_u64() || value.is_i64() => value.to_string(),
                _ => String::new(),
            })
            .unwrap_or_default();
        // A successful start is active even when all optional metadata is malformed.
        Ok(Self {
            live_key,
            protocols: parse_protocols(&data),
        })
    }
}

fn string_field(value: &Value, name: &str) -> String {
    value
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn parse_protocol(value: &Value, backup: bool, legacy: bool) -> Option<PushProtocol> {
    if !value.is_object() {
        return None;
    }
    let protocol = if legacy {
        "rtmp".to_owned()
    } else {
        string_field(value, "protocol")
    };
    if protocol.is_empty() {
        return None;
    }
    Some(PushProtocol {
        protocol,
        addr: string_field(value, "addr"),
        code: string_field(value, "code"),
        provider: string_field(value, "provider"),
        backup,
    })
}

pub fn parse_protocols(data: &Value) -> Vec<PushProtocol> {
    let mut result = Vec::new();
    for (field, legacy, backup) in [
        ("protocols", "rtmp", false),
        ("protocols_backup", "rtmp_backup", true),
    ] {
        let start = result.len();
        if let Some(values) = data.get(field).and_then(Value::as_array) {
            result.extend(
                values
                    .iter()
                    .filter_map(|value| parse_protocol(value, backup, false)),
            );
        }
        if result.len() == start
            && let Some(protocol) = data
                .get(legacy)
                .and_then(|value| parse_protocol(value, backup, true))
        {
            result.push(protocol);
        }
    }
    result
}
