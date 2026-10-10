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

fn validate_document(value: &Value) -> Result<(), String> {
    let version = value
        .get("contract_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "contract_version must be an integer".to_string())?;
    let _ = version;
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
                .iter()
                .find(|(name, _)| name.as_str() == key)
                .and_then(|(_, v)| v.as_array())
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
    if let Some(data_type) = envelope
        .iter()
        .find(|(key, _)| key.as_str() == "data_type")
        .map(|(_, v)| v)
    {
        if data_type.as_str() != Some("object_or_null") {
            return Err("unsupported envelope data_type".into());
        }
    }
    if let Some(canonical_omit) = envelope
        .iter()
        .find(|(key, _)| key.as_str() == "canonical_omit")
        .map(|(_, value)| value)
    {
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
        for key in ["category", "replay", "hash"] {
            if spec
                .iter()
                .find(|(name, _)| name.as_str() == key)
                .and_then(|(_, v)| v.as_str())
                .is_none()
            {
                return Err(format!("{}.{} must be a string", event_type, key));
            }
        }
        let replay = spec
            .iter()
            .find(|(key, _)| key.as_str() == "replay")
            .and_then(|(_, v)| v.as_str())
            .unwrap();
        let hash = spec
            .iter()
            .find(|(key, _)| key.as_str() == "hash")
            .and_then(|(_, v)| v.as_str())
            .unwrap();
        if !matches!(replay, "none" | "reissue" | "skip") || !matches!(hash, "noise" | "semantic") {
            return Err(format!(
                "{} has an unsupported replay/hash policy",
                event_type
            ));
        }
        let projection = spec
            .iter()
            .find(|(key, _)| key.as_str() == "projection")
            .and_then(|(_, v)| v.as_array())
            .ok_or_else(|| format!("{}.projection must be an array", event_type))?;
        if projection.iter().any(|item| item.as_str().is_none()) {
            return Err(format!("{}.projection must contain strings", event_type));
        }
        if let Some(policy_value) = spec
            .iter()
            .find(|(key, _)| key.as_str() == "data_policy")
            .map(|(_, value)| value)
        {
            let policy = policy_value
                .as_str()
                .ok_or_else(|| format!("{}.data_policy must be a string", event_type))?;
            if policy != "opaque_object" {
                return Err(format!("{}.data_policy is unsupported", event_type));
            }
        }
        let data_required = if spec.iter().any(|(key, _)| key.as_str() == "data_required") {
            strings(spec, "data_required")?
        } else {
            Vec::new()
        };
        let data_types_value = spec
            .iter()
            .find(|(key, _)| key.as_str() == "data_types")
            .map(|(_, value)| value);
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
                if !data_required.contains(field)
                    || !matches!(
                        kind.as_str(),
                        Some(
                            "string" | "number" | "number_or_null" | "boolean" | "object" | "array"
                        )
                    )
                {
                    return Err(format!("{}.data_types is malformed", event_type));
                }
            }
        }
        let nested_value = spec
            .iter()
            .find(|(key, _)| key.as_str() == "nested")
            .map(|(_, value)| value);
        if let Some(nested_value) = nested_value {
            let nested = nested_value
                .as_object()
                .ok_or_else(|| format!("{}.nested must be an object", event_type))?;
            for (field, nested_spec) in nested {
                let is_object = data_types
                    .and_then(|types| types.iter().find(|(key, _)| key.as_str() == field.as_str()))
                    .and_then(|(_, value)| value.as_str())
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
                    .iter()
                    .find(|(key, _)| key.as_str() == "types")
                    .and_then(|(_, value)| value.as_object())
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
                    if !matches!(
                        kind.as_str(),
                        Some(
                            "string" | "number" | "number_or_null" | "boolean" | "object" | "array"
                        )
                    ) {
                        return Err(format!(
                            "{}.nested.{} has an unsupported type",
                            event_type, field
                        ));
                    }
                }
                if let Some(items) = nested_spec
                    .iter()
                    .find(|(key, _)| key.as_str() == "items")
                    .map(|(_, value)| value)
                {
                    let items = items.as_object().ok_or_else(|| {
                        format!("{}.nested.{}.items must be an object", event_type, field)
                    })?;
                    for (item_field, item_spec) in items {
                        let is_array = nested_types
                            .iter()
                            .find(|(key, _)| key.as_str() == item_field.as_str())
                            .and_then(|(_, value)| value.as_str())
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
                        let item_required =
                            if item_spec.iter().any(|(key, _)| key.as_str() == "required") {
                                strings(item_spec, "required")?
                            } else {
                                Vec::new()
                            };
                        let item_types_value = item_spec
                            .iter()
                            .find(|(key, _)| key.as_str() == "types")
                            .map(|(_, value)| value);
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
                            if !matches!(
                                kind.as_str(),
                                Some(
                                    "string"
                                        | "number"
                                        | "number_or_null"
                                        | "boolean"
                                        | "object"
                                        | "array"
                                )
                            ) {
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

pub fn data_has_required_fields(event_type: &str, data: &crate::json::Value) -> bool {
    let contract = document();
    let required = contract
        .get("events")
        .and_then(Value::as_object)
        .and_then(|events| events.get(event_type))
        .and_then(|spec| spec.get("data_required"))
        .and_then(Value::as_array);
    let Some(required) = required else {
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
    let Some(types) = contract
        .get("events")
        .and_then(Value::as_object)
        .and_then(|events| events.get(event_type))
        .and_then(|spec| spec.get("data_types"))
        .and_then(Value::as_object)
    else {
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
    let nested = contract
        .get("events")
        .and_then(Value::as_object)
        .and_then(|events| events.iter().find(|(key, _)| key.as_str() == event_type))
        .and_then(|(_, spec)| spec.get("nested"))
        .and_then(Value::as_object);
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
    if let Some(required) = spec
        .iter()
        .find(|(key, _)| key.as_str() == "required")
        .and_then(|(_, value)| value.as_array())
    {
        for field in required {
            let Some(field) = field.as_str() else {
                return false;
            };
            if object.iter().all(|(key, _)| key != field) {
                return false;
            }
        }
    }
    if let Some(types) = spec
        .iter()
        .find(|(key, _)| key.as_str() == "types")
        .and_then(|(_, value)| value.as_object())
    {
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
    if let Some(items) = spec
        .iter()
        .find(|(key, _)| key.as_str() == "items")
        .and_then(|(_, value)| value.as_object())
    {
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
        assert_eq!(contract_version(), 1);
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
            let event_type = case["event"]["type"].as_str().unwrap();
            let data = crate::json::parse(&case["event"]["data"].to_string()).unwrap();
            let expected = case["data_valid"].as_bool().unwrap();
            assert_eq!(
                data_has_required_fields(event_type, &data),
                expected,
                "{}",
                case["name"]
            );
        }
    }
}
