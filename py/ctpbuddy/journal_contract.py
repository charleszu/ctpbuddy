"""Canonical journal event contract shared by Rust and Python adapters."""
from __future__ import annotations

import json
import hashlib
import math
from pathlib import Path
from typing import Any, Dict, FrozenSet, List


_PATH = Path(__file__).with_name("contracts") / "journal_contract.json"


def _load() -> Dict[str, Any]:
    with _PATH.open("r", encoding="utf-8") as fh:
        value = json.load(fh)
    if not isinstance(value, dict) or not isinstance(value.get("contract_version"), int):
        raise RuntimeError("journal contract must contain an integer contract_version")
    if not isinstance(value.get("envelope"), dict) or not isinstance(value.get("events"), dict):
        raise RuntimeError("journal contract must contain envelope and events")
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
    if "contract_version" in event and event["contract_version"] != CONTRACT_VERSION:
        problems.append("contract_version %r does not match %d" % (event["contract_version"], CONTRACT_VERSION))
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
    for key in ("ts_wall", "trading_day", "type"):
        if key in event and not isinstance(event[key], str):
            problems.append("%s must be a string" % key)
    if "seq" in event and (isinstance(event["seq"], bool) or not isinstance(event["seq"], int)):
        problems.append("seq must be an integer")
    if "vt_ms" in event and (isinstance(event["vt_ms"], bool) or not isinstance(event["vt_ms"], (int, float))):
        problems.append("vt_ms must be a number")
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
    return True
