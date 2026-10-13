//! Shared journal contract embedded from the package artifact.

use serde_json::Value;
use std::sync::OnceLock;

pub const CONTRACT_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/journal_contract.json"));

pub fn document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        let value: Value = serde_json::from_str(CONTRACT_JSON)
            .expect("embedded journal contract must be valid JSON");
        validate_document(&value).expect("embedded journal contract is malformed");
        value
    })
}

/// The contract's closed set of data types (see data_types / nested.types).
fn known_type(kind: Option<&str>) -> bool {
    matches!(
        kind,
        Some("string" | "number" | "number_or_null" | "boolean" | "object" | "array")
    )
}

fn validate_document(value: &Value) -> Result<(), String> {
    let version = value
        .get("contract_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "contract_version must be an integer".to_string())?;
    if let Some(versions) = value.get("read_versions") {
        let versions = versions
            .as_array()
            .ok_or("read_versions must be an array")?;
        if !versions.iter().any(|v| v.as_u64() == Some(version))
            || versions
                .iter()
                .any(|v| v.as_u64().is_none_or(|v| v == 0 || v > version))
        {
            return Err("read_versions must contain supported integer versions".into());
        }
    }
    let envelope = value
        .get("envelope")
        .and_then(Value::as_object)
        .ok_or_else(|| "envelope must be an object".to_string())?;
    let events = value
        .get("events")
        .and_then(Value::as_object)
        .ok_or_else(|| "events must be an object".to_string())?;
    if events.is_empty() {
        return Err("events must not be empty".into());
    }
    let strings =
        |object: &serde_json::Map<String, Value>, key: &str| -> Result<Vec<String>, String> {
            let values = object
                .get(key)
                .and_then(Value::as_array)
                .ok_or_else(|| format!("{} must be an array", key))?;
            values
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| format!("{} must contain strings", key))
                })
                .collect()
        };
    let required = strings(envelope, "required")?;
    let optional = strings(envelope, "optional")?;
    if required.iter().any(|key| optional.contains(key)) {
        return Err("envelope required/optional fields overlap".into());
    }
    if let Some(data_type) = envelope.get("data_type") {
        if data_type.as_str() != Some("object_or_null") {
            return Err("unsupported envelope data_type".into());
        }
    }
    if let Some(canonical_omit) = envelope.get("canonical_omit") {
        let canonical_omit = canonical_omit
            .as_array()
            .ok_or_else(|| "canonical_omit must be an array".to_string())?;
        if canonical_omit.iter().any(|item| item.as_str().is_none()) {
            return Err("canonical_omit must contain strings".into());
        }
    }
    for (event_type, spec) in events {
        let spec = spec
            .as_object()
            .ok_or_else(|| format!("{} must be an object", event_type))?;
        if spec
            .get("min_version")
            .is_some_and(|v| v.as_u64().is_none_or(|v| v == 0 || v > version))
        {
            return Err("event min_version is invalid".into());
        }
        for key in ["category", "replay", "hash"] {
            if spec.get(key).and_then(Value::as_str).is_none() {
                return Err(format!("{}.{} must be a string", event_type, key));
            }
        }
        let replay = spec.get("replay").and_then(Value::as_str).unwrap();
        let hash = spec.get("hash").and_then(Value::as_str).unwrap();
        if !matches!(replay, "none" | "reissue" | "skip") || !matches!(hash, "noise" | "semantic") {
            return Err(format!(
                "{} has an unsupported replay/hash policy",
                event_type
            ));
        }
        let projection = spec
            .get("projection")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{}.projection must be an array", event_type))?;
        if projection.iter().any(|item| item.as_str().is_none()) {
            return Err(format!("{}.projection must contain strings", event_type));
        }
        if let Some(policy_value) = spec.get("data_policy") {
            let policy = policy_value
                .as_str()
                .ok_or_else(|| format!("{}.data_policy must be a string", event_type))?;
            if policy != "opaque_object" {
                return Err(format!("{}.data_policy is unsupported", event_type));
            }
        }
        let data_required = if spec.contains_key("data_required") {
            strings(spec, "data_required")?
        } else {
            Vec::new()
        };
        let data_types_value = spec.get("data_types");
        let data_types = match data_types_value {
            Some(value) => Some(
                value
                    .as_object()
                    .ok_or_else(|| format!("{}.data_types must be an object", event_type))?,
            ),
            None => None,
        };
        if let Some(types) = data_types {
            for (field, kind) in types {
                if !data_required.contains(field) || !known_type(kind.as_str()) {
                    return Err(format!("{}.data_types is malformed", event_type));
                }
            }
        }
        let nested_value = spec.get("nested");
        if let Some(nested_value) = nested_value {
            let nested = nested_value
                .as_object()
                .ok_or_else(|| format!("{}.nested must be an object", event_type))?;
            for (field, nested_spec) in nested {
                let is_object = data_types
                    .and_then(|types| types.get(field.as_str()))
                    .and_then(Value::as_str)
                    == Some("object");
                if !data_required.contains(field) || !is_object {
                    return Err(format!(
                        "{}.nested.{} must be a required object",
                        event_type, field
                    ));
                }
                let nested_spec = nested_spec
                    .as_object()
                    .ok_or_else(|| format!("{}.nested.{} must be an object", event_type, field))?;
                let nested_required = strings(nested_spec, "required")?;
                let nested_types = nested_spec
                    .get("types")
                    .and_then(Value::as_object)
                    .ok_or_else(|| {
                        format!("{}.nested.{}.types must be an object", event_type, field)
                    })?;
                for required_field in nested_required {
                    if !nested_types.iter().any(|(key, _)| key == &required_field) {
                        return Err(format!(
                            "{}.nested.{}.types misses {}",
                            event_type, field, required_field
                        ));
                    }
                }
                for (_, kind) in nested_types {
                    if !known_type(kind.as_str()) {
                        return Err(format!(
                            "{}.nested.{} has an unsupported type",
                            event_type, field
                        ));
                    }
                }
                if let Some(items) = nested_spec.get("items") {
                    let items = items.as_object().ok_or_else(|| {
                        format!("{}.nested.{}.items must be an object", event_type, field)
                    })?;
                    for (item_field, item_spec) in items {
                        let is_array = nested_types
                            .get(item_field.as_str())
                            .and_then(Value::as_str)
                            == Some("array");
                        if !is_array {
                            return Err(format!(
                                "{}.nested.{}.items.{} must target an array",
                                event_type, field, item_field
                            ));
                        }
                        let item_spec = item_spec.as_object().ok_or_else(|| {
                            format!(
                                "{}.nested.{}.items.{} must be an object",
                                event_type, field, item_field
                            )
                        })?;
                        let item_required = if item_spec.contains_key("required") {
                            strings(item_spec, "required")?
                        } else {
                            Vec::new()
                        };
                        let item_types_value = item_spec.get("types");
                        let item_types = match item_types_value {
                            Some(value) => Some(value.as_object().ok_or_else(|| {
                                format!(
                                    "{}.nested.{}.items.{}.types must be an object",
                                    event_type, field, item_field
                                )
                            })?),
                            None => None,
                        };
                        let Some(item_types) = item_types else {
                            return Err(format!(
                                "{}.nested.{}.items.{} must declare types",
                                event_type, field, item_field
                            ));
                        };
                        for required_field in item_required {
                            if !item_types.iter().any(|(key, _)| key == &required_field) {
                                return Err(format!(
                                    "{}.nested.{}.items.{}.types misses {}",
                                    event_type, field, item_field, required_field
                                ));
                            }
                        }
                        for (_, kind) in item_types {
                            if !known_type(kind.as_str()) {
                                return Err(format!(
                                    "{}.nested.{}.items.{} has an unsupported type",
                                    event_type, field, item_field
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn contract_version() -> u64 {
    document()
        .get("contract_version")
        .and_then(Value::as_u64)
        .expect("journal contract_version must be an integer")
}

pub fn knows_event(event_type: &str) -> bool {
    document()
        .get("events")
        .and_then(Value::as_object)
        .is_some_and(|events| events.contains_key(event_type))
}

/// Validate a complete journal envelope before the writer advances its seq.
/// The server constructs these envelopes itself, but keeping this check here
/// makes the Rust adapter obey the same fail-closed contract as Python.
pub fn validate_event(event: &crate::json::Value) -> bool {
    let Some(object) = event.as_obj() else {
        return false;
    };
    let Some(envelope) = document().get("envelope").and_then(Value::as_object) else {
        return false;
    };
    let required = envelope.get("required").and_then(Value::as_array);
    let optional = envelope.get("optional").and_then(Value::as_array);
    let Some(required) = required else {
        return false;
    };
    let Some(optional) = optional else {
        return false;
    };
    let allowed = |key: &str| {
        key == "contract_version"
            || required.iter().any(|item| item.as_str() == Some(key))
            || optional.iter().any(|item| item.as_str() == Some(key))
    };
    if object.iter().any(|(key, _)| !allowed(key))
        || required.iter().any(|field| {
            field
                .as_str()
                .is_none_or(|name| object.iter().all(|(key, _)| key != name))
        })
    {
        return false;
    }
    if let Some(version) = event.get("contract_version") {
        let Some(version) = version.as_num() else {
            return false;
        };
        let supported = document()
            .get("read_versions")
            .and_then(Value::as_array)
            .map(|versions| {
                versions
                    .iter()
                    .any(|v| v.as_u64().is_some_and(|v| v as f64 == version))
            })
            .unwrap_or(version == contract_version() as f64);
        let minimum = event
            .get("type")
            .and_then(crate::json::Value::as_str)
            .and_then(|name| {
                document()
                    .get("events")?
                    .get(name)?
                    .get("min_version")?
                    .as_u64()
            })
            .unwrap_or(1);
        if !version.is_finite() || version.fract() != 0.0 || !supported || version < minimum as f64
        {
            return false;
        }
    }
    let Some(event_type) = event.get("type").and_then(crate::json::Value::as_str) else {
        return false;
    };
    if !knows_event(event_type) {
        return false;
    }
    for key in ["ts_wall", "trading_day", "type"] {
        if event
            .get(key)
            .and_then(crate::json::Value::as_str)
            .is_none()
        {
            return false;
        }
    }
    let Some(seq) = event.get("seq").and_then(crate::json::Value::as_num) else {
        return false;
    };
    let Some(vt_ms) = event.get("vt_ms").and_then(crate::json::Value::as_num) else {
        return false;
    };
    if !seq.is_finite() || seq.fract() != 0.0 || !vt_ms.is_finite() {
        return false;
    }
    for key in optional {
        let Some(key) = key.as_str() else {
            return false;
        };
        if event.get(key).is_some_and(|value| value.as_str().is_none()) {
            return false;
        }
    }
    let Some(data) = event.get("data") else {
        return false;
    };
    if !matches!(data, crate::json::Value::Obj(_) | crate::json::Value::Null) {
        return false;
    }
    data_has_required_fields(event_type, data)
}

pub fn data_has_required_fields(event_type: &str, data: &crate::json::Value) -> bool {
    let Some(spec) = document()
        .get("events")
        .and_then(Value::as_object)
        .and_then(|events| events.get(event_type))
    else {
        return true;
    };
    let Some(required) = spec.get("data_required").and_then(Value::as_array) else {
        return true;
    };
    let Some(object) = data.as_obj() else {
        return false;
    };
    if !required.iter().all(|field| {
        field
            .as_str()
            .is_some_and(|name| object.iter().any(|(key, _)| key == name))
    }) {
        return false;
    }
    let Some(types) = spec.get("data_types").and_then(Value::as_object) else {
        return true;
    };
    if !types.iter().all(|(field, kind)| {
        let Some(value) = object
            .iter()
            .find(|(key, _)| key == field)
            .map(|(_, value)| value)
        else {
            return false;
        };
        let Some(kind) = kind.as_str() else {
            return false;
        };
        matches_data_type(value, kind)
    }) {
        return false;
    }
    let nested = spec.get("nested").and_then(Value::as_object);
    if let Some(nested) = nested {
        for (field, nested_spec) in nested {
            let Some(value) = object
                .iter()
                .find(|(key, _)| key == field)
                .map(|(_, value)| value)
            else {
                return false;
            };
            if !matches_nested(value, nested_spec) {
                return false;
            }
        }
    }
    true
}

fn matches_nested(value: &crate::json::Value, spec: &Value) -> bool {
    let Some(object) = value.as_obj() else {
        return false;
    };
    let Some(spec) = spec.as_object() else {
        return false;
    };
    if let Some(required) = spec.get("required").and_then(Value::as_array) {
        for field in required {
            let Some(field) = field.as_str() else {
                return false;
            };
            if object.iter().all(|(key, _)| key != field) {
                return false;
            }
        }
    }
    if let Some(types) = spec.get("types").and_then(Value::as_object) {
        for (field, kind) in types {
            let Some(value) = object
                .iter()
                .find(|(key, _)| key == field)
                .map(|(_, value)| value)
            else {
                continue;
            };
            let Some(kind) = kind.as_str() else {
                return false;
            };
            if !matches_data_type(value, kind) {
                return false;
            }
        }
    }
    if let Some(items) = spec.get("items").and_then(Value::as_object) {
        for (field, item_spec) in items {
            let Some(values) =
                object
                    .iter()
                    .find(|(key, _)| key == field)
                    .and_then(|(_, value)| match value {
                        crate::json::Value::Arr(values) => Some(values),
                        _ => None,
                    })
            else {
                continue;
            };
            for value in values {
                if !matches_nested(value, item_spec) {
                    return false;
                }
            }
        }
    }
    true
}

fn matches_data_type(value: &crate::json::Value, kind: &str) -> bool {
    match (kind, value) {
        ("string", crate::json::Value::Str(_)) => true,
        ("number", crate::json::Value::Num(n)) => n.is_finite(),
        ("number_or_null", crate::json::Value::Null) => true,
        ("number_or_null", crate::json::Value::Num(n)) => n.is_finite(),
        ("boolean", crate::json::Value::Bool(_)) => true,
        ("object", crate::json::Value::Obj(_)) => true,
        ("array", crate::json::Value::Arr(_)) => true,
        _ => false,
    }
}

/// Canonical JSON used by the shared vector tests (sorted keys, wall clock omitted).
#[cfg(test)]
pub fn canonical_event(event: &Value) -> String {
    let mut normalized = event.clone();
    if let Some(object) = normalized.as_object_mut() {
        object.remove("ts_wall");
    }
    fn write(value: &Value, out: &mut String) {
        match value {
            Value::Object(object) => {
                out.push('{');
                for (index, (key, value)) in object
                    .iter()
                    .collect::<std::collections::BTreeMap<_, _>>()
                    .into_iter()
                    .enumerate()
                {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(&serde_json::to_string(key).unwrap());
                    out.push(':');
                    write(value, out);
                }
                out.push('}');
            }
            Value::Array(values) => {
                out.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write(value, out);
                }
                out.push(']');
            }
            _ => out.push_str(&serde_json::to_string(value).unwrap()),
        }
    }
    let mut output = String::new();
    write(&normalized, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_contract_declares_pilot_events() {
        assert_eq!(contract_version(), 3);
        assert!(knows_event("server_start"));
        assert!(knows_event("session_auth"));
        assert!(!knows_event("made_up_event"));
        assert!(data_has_required_fields(
            "fill",
            &crate::json::Value::Obj(vec![
                ("trade_id".into(), crate::json::Value::Str("t".into())),
                ("instrument".into(), crate::json::Value::Str("rb".into())),
                ("direction".into(), crate::json::Value::Num(48.0)),
                ("offset".into(), crate::json::Value::Num(48.0)),
                ("price".into(), crate::json::Value::Num(1.0)),
                ("volume".into(), crate::json::Value::Num(1.0)),
            ])
        ));
        assert!(data_has_required_fields(
            "order_insert",
            &crate::json::Value::Obj(vec![
                ("order_ref".into(), crate::json::Value::Str("r".into())),
                ("instrument".into(), crate::json::Value::Str("rb".into())),
                ("exchange".into(), crate::json::Value::Str("SHFE".into())),
                ("direction".into(), crate::json::Value::Num(48.0)),
                ("offset".into(), crate::json::Value::Num(48.0)),
                ("price_type".into(), crate::json::Value::Str("2".into())),
                ("limit_price".into(), crate::json::Value::Num(1.0)),
                ("volume".into(), crate::json::Value::Num(1.0)),
                ("time_condition".into(), crate::json::Value::Str("3".into())),
                (
                    "volume_condition".into(),
                    crate::json::Value::Str("1".into())
                ),
                ("min_volume".into(), crate::json::Value::Num(1.0)),
                (
                    "outcome".into(),
                    crate::json::Value::Obj(vec![
                        ("accepted".into(), crate::json::Value::Bool(true)),
                        (
                            "fills".into(),
                            crate::json::Value::Arr(vec![crate::json::Value::Obj(vec![
                                ("trade_id".into(), crate::json::Value::Str("t".into())),
                                ("price".into(), crate::json::Value::Num(1.0)),
                                ("volume".into(), crate::json::Value::Num(1.0)),
                            ])])
                        ),
                    ])
                ),
            ])
        ));
    }

    #[test]
    fn canonical_vector_matches_python_adapter() {
        let vectors: Value = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/journal_vectors.json"
        )))
        .unwrap();
        let vector = &vectors["canonical"][0];
        assert_eq!(canonical_event(&vector["event"]), vector["expected"]);
    }

    #[test]
    fn validation_vectors_match_python_adapter() {
        let vectors: Value = serde_json::from_str(include_str!(concat!(
            env!("OUT_DIR"),
            "/journal_vectors.json"
        )))
        .unwrap();
        for case in vectors["validation"].as_array().unwrap() {
            let event = crate::json::parse(&case["event"].to_string()).unwrap();
            let expected = case["event_valid"].as_bool().unwrap();
            assert_eq!(validate_event(&event), expected, "{}", case["name"]);
        }
    }
}
