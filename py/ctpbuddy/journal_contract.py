"""Canonical journal event contract shared by Rust and Python adapters."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict, FrozenSet


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
EVENTS: Dict[str, Dict[str, Any]] = CONTRACT["events"]
ENVELOPE_REQUIRED: tuple[str, ...] = tuple(CONTRACT["envelope"]["required"])
ENVELOPE_OPTIONAL: tuple[str, ...] = tuple(CONTRACT["envelope"]["optional"])
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

