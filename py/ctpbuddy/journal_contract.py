"""Canonical journal event contract shared by Rust and Python adapters."""
from __future__ import annotations

import json
import hashlib
import math
from pathlib import Path
from typing import Any, Dict, FrozenSet, List


_PATH = Path(__file__).with_name("contracts") / "journal_contract.json"
_DATA_TYPES = frozenset({"string", "number", "number_or_null", "boolean", "object", "array"})
_DATA_POLICIES = frozenset({"opaque_object"})
_REPLAYS = frozenset({"none", "reissue", "skip"})
_HASHES = frozenset({"noise", "semantic"})


def _validate_contract_document(value: Dict[str, Any]) -> None:
    """Fail fast when the shared artifact itself is malformed."""
    version = value.get("contract_version")
    if isinstance(version, bool) or not isinstance(version, int):
        raise RuntimeError("journal contract must contain an integer contract_version")
    envelope = value.get("envelope")
    events = value.get("events")
    if not isinstance(envelope, dict) or not isinstance(events, dict) or not events:
        raise RuntimeError("journal contract must contain non-empty envelope and events")
    required = envelope.get("required")
    optional = envelope.get("optional")
    if not isinstance(required, list) or not all(isinstance(item, str) for item in required):
        raise RuntimeError("journal contract envelope.required must be an array of strings")
    if not isinstance(optional, list) or not all(isinstance(item, str) for item in optional):
        raise RuntimeError("journal contract envelope.optional must be an array of strings")
    if set(required) & set(optional):
        raise RuntimeError("journal contract envelope required/optional fields overlap")
    data_type = envelope.get("data_type", "object_or_null")
    if data_type != "object_or_null":
        raise RuntimeError("journal contract envelope.data_type is unsupported: %r" % data_type)
    omitted = envelope.get("canonical_omit", [])
    if not isinstance(omitted, list) or not all(isinstance(item, str) for item in omitted):
        raise RuntimeError("journal contract envelope.canonical_omit must be an array of strings")
    for event_type, spec in events.items():
        if not isinstance(event_type, str) or not isinstance(spec, dict):
            raise RuntimeError("journal contract event specs must be objects")
        for key in ("category", "replay", "hash"):
            if not isinstance(spec.get(key), str):
                raise RuntimeError("journal contract %s.%s must be a string" % (event_type, key))
        if spec["replay"] not in _REPLAYS or spec["hash"] not in _HASHES:
            raise RuntimeError("journal contract %s has an unsupported replay/hash policy" % event_type)
        projection = spec.get("projection")
        if not isinstance(projection, list) or not all(isinstance(item, str) for item in projection):
            raise RuntimeError("journal contract %s.projection must be an array of strings" % event_type)
        if "data_policy" in spec and spec["data_policy"] not in _DATA_POLICIES:
            raise RuntimeError("journal contract %s.data_policy is unsupported" % event_type)
        data_required = spec.get("data_required", [])
        data_types = spec.get("data_types", {})
        if not isinstance(data_required, list) or not all(isinstance(item, str) for item in data_required):
            raise RuntimeError("journal contract %s.data_required must be an array of strings" % event_type)
        if not isinstance(data_types, dict):
            raise RuntimeError("journal contract %s.data_types must be an object" % event_type)
        if not set(data_types).issubset(data_required):
            raise RuntimeError("journal contract %s.data_types contains non-required fields" % event_type)
        if any(kind not in _DATA_TYPES for kind in data_types.values()):
            raise RuntimeError("journal contract %s has an unsupported data type" % event_type)
        nested = spec.get("nested", {})
        if not isinstance(nested, dict):
            raise RuntimeError("journal contract %s.nested must be an object" % event_type)
        for field, nested_spec in nested.items():
            if field not in data_required or data_types.get(field) != "object":
                raise RuntimeError("journal contract %s nested field %s must be a required object" % (event_type, field))
            if not isinstance(nested_spec, dict):
                raise RuntimeError("journal contract %s.nested.%s must be an object" % (event_type, field))
            nested_required = nested_spec.get("required", [])
            nested_types = nested_spec.get("types", {})
            if not isinstance(nested_required, list) or not all(isinstance(item, str) for item in nested_required):
                raise RuntimeError("journal contract %s.nested.%s.required must be an array of strings" % (event_type, field))
            if not isinstance(nested_types, dict) or not set(nested_types).issuperset(nested_required):
                raise RuntimeError("journal contract %s.nested.%s.types must cover required fields" % (event_type, field))
            if any(kind not in _DATA_TYPES for kind in nested_types.values()):
                raise RuntimeError("journal contract %s.nested.%s has an unsupported type" % (event_type, field))
            items = nested_spec.get("items", {})
            if not isinstance(items, dict):
                raise RuntimeError("journal contract %s.nested.%s.items must be an object" % (event_type, field))
            for item_field, item_spec in items.items():
                if nested_types.get(item_field) != "array" or not isinstance(item_spec, dict):
                    raise RuntimeError("journal contract %s.nested.%s.items.%s is invalid" % (event_type, field, item_field))
                item_required = item_spec.get("required", [])
                item_types = item_spec.get("types")
                if not isinstance(item_required, list) or not isinstance(item_types, dict):
                    raise RuntimeError("journal contract %s.nested.%s.items.%s is malformed" % (event_type, field, item_field))
                if not set(item_types).issuperset(item_required) or any(kind not in _DATA_TYPES for kind in item_types.values()):
                    raise RuntimeError("journal contract %s.nested.%s.items.%s has invalid types" % (event_type, field, item_field))


def _load() -> Dict[str, Any]:
    with _PATH.open("r", encoding="utf-8") as fh:
        value = json.load(fh)
    if not isinstance(value, dict):
        raise RuntimeError("journal contract must be an object")
    _validate_contract_document(value)
    return value


CONTRACT: Dict[str, Any] = _load()
CONTRACT_VERSION: int = CONTRACT["contract_version"]
CONTRACT_SHA256: str = hashlib.sha256(_PATH.read_bytes()).hexdigest()
EVENTS: Dict[str, Dict[str, Any]] = CONTRACT["events"]
ENVELOPE_REQUIRED: tuple[str, ...] = tuple(CONTRACT["envelope"]["required"])
ENVELOPE_OPTIONAL: tuple[str, ...] = tuple(CONTRACT["envelope"]["optional"])
ENVELOPE_ALLOWED: FrozenSet[str] = frozenset(ENVELOPE_REQUIRED) | frozenset(ENVELOPE_OPTIONAL) | {"contract_version"}
DATA_TYPE: str = str(CONTRACT["envelope"].get("data_type", "object_or_null"))
CANONICAL_OMIT_FIELDS: FrozenSet[str] = frozenset(CONTRACT["envelope"].get("canonical_omit", []))
SESSION_FRONT_ID_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items() if "front_id_rank" in spec.get("normalize", [])
)
KNOWN_TYPES: FrozenSet[str] = frozenset(EVENTS)
CORE_NOISE_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items() if spec.get("hash") == "noise"
)
REPLAYABLE_ADMIN_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items()
    if spec.get("category") == "admin_input" and spec.get("replay") == "reissue"
)
UNREPLAYABLE_ADMIN_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items()
    if spec.get("category") == "admin_input" and spec.get("replay") == "skip"
)
ADMIN_INPUT_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items() if spec.get("category") == "admin_input"
)
# The walker must see both reissued and explicitly skipped admin inputs so it
# can report `not_replayed` instead of silently dropping them.
REPLAYABLE_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items() if spec.get("replay") == "reissue"
) | ADMIN_INPUT_TYPES
AUDIT_TYPES: FrozenSet[str] = frozenset(
    name for name, spec in EVENTS.items() if "audit_log" in spec.get("projection", [])
)
SETTLEMENT_TYPES: FrozenSet[str] = frozenset(
    name for name in ("settlement", "settle") if name in EVENTS
)


def event_spec(event_type: str) -> Dict[str, Any]:
    """Return a declared event spec or raise a contract error."""
    try:
        return EVENTS[event_type]
    except KeyError as exc:
        raise ValueError("unknown journal event type: %s" % event_type) from exc


def validate_event_contract(event: Dict[str, Any]) -> List[str]:
    """Return contract violations for one envelope, without checking seq order."""
    problems: List[str] = []
    extra = sorted(set(event) - ENVELOPE_ALLOWED)
    problems.extend("unknown envelope field %r" % key for key in extra)
    missing = [key for key in ENVELOPE_REQUIRED if key not in event]
    if missing:
        problems.extend("missing field %r" % key for key in missing)
    event_type = event.get("type")
    if not isinstance(event_type, str) or event_type not in KNOWN_TYPES:
        problems.append("unknown event type %r" % event_type)
    if "contract_version" in event:
        version = event["contract_version"]
        if isinstance(version, bool) or not isinstance(version, int) or version != CONTRACT_VERSION:
            problems.append("contract_version %r does not match %d" % (version, CONTRACT_VERSION))
    if "data" in event and DATA_TYPE == "object_or_null" and event["data"] is not None and not isinstance(event["data"], dict):
        problems.append("data must be an object or null")
    spec = EVENTS.get(event_type) if isinstance(event_type, str) else None
    required_data = spec.get("data_required", []) if spec else []
    data = event.get("data")
    if required_data:
        if not isinstance(data, dict):
            problems.append("data must be an object for %s" % event_type)
        else:
            problems.extend("data missing field %r" % key for key in required_data if key not in data)
    if isinstance(data, dict) and spec:
        for key, kind in spec.get("data_types", {}).items():
            if key not in data or not _matches_type(data[key], kind):
                problems.append("data field %r must be %s" % (key, kind))
        for field, nested_spec in spec.get("nested", {}).items():
            nested = data.get(field)
            if not isinstance(nested, dict):
                problems.append("data field %r must be object" % field)
                continue
            for key in nested_spec.get("required", []):
                if key not in nested:
                    problems.append("data field %r missing nested field %r" % (field, key))
            for key, kind in nested_spec.get("types", {}).items():
                if key in nested and not _matches_type(nested[key], kind):
                    problems.append("data field %r.%r must be %s" % (field, key, kind))
            for item_field, item_spec in nested_spec.get("items", {}).items():
                items = nested.get(item_field)
                if not isinstance(items, list):
                    continue
                for index, item in enumerate(items):
                    if not isinstance(item, dict):
                        problems.append("data field %r.%s[%d] must be object" % (field, item_field, index))
                        continue
                    for key in item_spec.get("required", []):
                        if key not in item:
                            problems.append("data field %r.%s[%d] missing field %r" % (field, item_field, index, key))
                    for key, kind in item_spec.get("types", {}).items():
                        if key in item and not _matches_type(item[key], kind):
                            problems.append("data field %r.%s[%d].%r must be %s" % (field, item_field, index, key, kind))
    for key in ("ts_wall", "trading_day", "type"):
        if key in event and not isinstance(event[key], str):
            problems.append("%s must be a string" % key)
    if "seq" in event and (isinstance(event["seq"], bool) or not isinstance(event["seq"], int)):
        problems.append("seq must be an integer")
    if "vt_ms" in event:
        if isinstance(event["vt_ms"], bool) or not isinstance(event["vt_ms"], (int, float)):
            problems.append("vt_ms must be a number")
        elif not math.isfinite(event["vt_ms"]):
            problems.append("vt_ms must be finite")
    for key in ENVELOPE_OPTIONAL:
        if key in event and not isinstance(event[key], str):
            problems.append("%s must be a string" % key)
    return problems


def _matches_type(value: Any, kind: str) -> bool:
    if kind == "string":
        return isinstance(value, str)
    if kind == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)
    if kind == "number_or_null":
        return value is None or _matches_type(value, "number")
    if kind == "boolean":
        return isinstance(value, bool)
    if kind == "object":
        return isinstance(value, dict)
    if kind == "array":
        return isinstance(value, list)
    return False
