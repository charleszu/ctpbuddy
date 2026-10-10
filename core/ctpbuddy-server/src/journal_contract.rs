//! Shared journal contract embedded from the package artifact.

use serde_json::Value;

pub const CONTRACT_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../py/ctpbuddy/contracts/journal_contract.json"
));

pub fn document() -> Value {
    serde_json::from_str(CONTRACT_JSON).expect("embedded journal contract must be valid JSON")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_contract_declares_pilot_events() {
        assert_eq!(contract_version(), 1);
        assert!(knows_event("server_start"));
        assert!(knows_event("session_auth"));
        assert!(!knows_event("made_up_event"));
    }
}
