//! Shared journal contract embedded from the package artifact.

use serde_json::Value;
use std::sync::OnceLock;

pub const CONTRACT_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../py/ctpbuddy/contracts/journal_contract.json"
));

pub fn document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        serde_json::from_str(CONTRACT_JSON).expect("embedded journal contract must be valid JSON")
    })
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
    required.iter().all(|field| {
        field
            .as_str()
            .is_some_and(|name| object.iter().any(|(key, _)| key == name))
    })
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
                ("trade_id".into(), crate::json::Value::Null),
                ("instrument".into(), crate::json::Value::Null),
                ("direction".into(), crate::json::Value::Null),
                ("offset".into(), crate::json::Value::Null),
                ("price".into(), crate::json::Value::Null),
                ("volume".into(), crate::json::Value::Null),
            ])
        ));
    }

    #[test]
    fn canonical_vector_matches_python_adapter() {
        let vectors: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../py/ctpbuddy/contracts/journal_vectors.json"
        )))
        .unwrap();
        let vector = &vectors["canonical"][0];
        assert_eq!(canonical_event(&vector["event"]), vector["expected"]);
    }
}
