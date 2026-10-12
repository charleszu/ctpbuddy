"""Projection upgrade through the journal/read-model seam."""
import json
from contextlib import closing
from pathlib import Path
import sqlite3
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "py"))
from ctpbuddy.journal import JournalError
from ctpbuddy.journal_contract import CONTRACT_VERSION
from ctpbuddy.store import Projection, rebuild, upgrade


class UpgradeTests(unittest.TestCase):
    def test_upgrade_preserves_old_projection_and_is_idempotent(self):
        with tempfile.TemporaryDirectory() as root:
            journal, db = Path(root) / "events.jsonl", Path(root) / "old.db"
            event = dict(seq=1, ts_wall="audit", trading_day="20261012", vt_ms=0,
                         type="deposit", broker="8888", investor="u", data={"amount": 100})
            journal.write_text(json.dumps(event) + "\n", encoding="utf-8")
            rebuild(journal, db)
            with closing(sqlite3.connect(db)) as conn:
                conn.execute("UPDATE projection_meta SET value='1' WHERE key='contract_version'")
                conn.execute("UPDATE projection_meta SET value=? WHERE key='contract_sha256'", ("a" * 64,))
                conn.commit()
            result = upgrade(journal, db)
            self.assertTrue(result["upgraded"])
            with closing(sqlite3.connect(result["backup"])) as saved:
                self.assertEqual(saved.execute("SELECT value FROM projection_meta WHERE key='contract_version'").fetchone()[0], "1")
                self.assertEqual(saved.execute("SELECT action FROM audit_log").fetchone()[0], "deposit")
            with Projection(db) as projection:
                self.assertEqual(projection.metadata()["contract_version"], str(CONTRACT_VERSION))
                self.assertEqual(projection.query("audit_log")[0]["action"], "deposit")
            self.assertFalse(upgrade(journal, db)["upgraded"])
            # A different recording must never silently replace this projection.
            event["data"]["amount"] = 200
            journal.write_text(json.dumps(event) + "\n", encoding="utf-8")
            current = db.read_bytes()
            with self.assertRaises(JournalError):
                upgrade(journal, db)
            self.assertEqual(db.read_bytes(), current)

    def test_invalid_journal_preserves_projection_without_creating_backup(self):
        with tempfile.TemporaryDirectory() as root:
            journal, db = Path(root) / "events.jsonl", Path(root) / "old.db"
            journal.write_text(json.dumps(dict(seq=1, ts_wall="audit", trading_day="20261012",
                vt_ms=0, type="deposit", data={"amount": 100})) + "\n", encoding="utf-8")
            rebuild(journal, db)
            current = db.read_bytes()
            journal.write_text("{invalid}\n", encoding="utf-8")
            with self.assertRaises(JournalError):
                upgrade(journal, db)
            self.assertEqual(db.read_bytes(), current)
            self.assertEqual(list(Path(root).glob("*.bak")), [])


if __name__ == "__main__":
    unittest.main()
