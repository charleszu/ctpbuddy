from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path

from m3_refdata import BROKER, find_core, free_port, wait_port
from ctpbuddy.sdk import Admin, Client
from ctpbuddy.sdk.client import CTPError
from ctpbuddy.journal import load_events
from ctpbuddy.store import rebuild


def main():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        previous = None
        for iteration in range(2):
            td_port, admin_port = free_port(), free_port()
            command = [find_core(), "--td", f"127.0.0.1:{td_port}", "--admin",
                       f"127.0.0.1:{admin_port}", "--data-dir", directory, "--qry-freq", "100"]
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            try:
                wait_port(admin_port)
                with Admin(f"127.0.0.1:{admin_port}") as admin:
                    competing = command.copy()
                    competing[2] = f"127.0.0.1:{free_port()}"
                    competing[4] = f"127.0.0.1:{free_port()}"
                    result = subprocess.run(competing, capture_output=True, timeout=10)
                    assert result.returncode != 0, "shared data directory must reject a second writer"
                    with Client(f"127.0.0.1:{td_port}") as client:
                        client.auth(BROKER, "review")
                        client.login(BROKER, "review")
                        client.settle_confirm()
                        before = client.qry_trading_account()
                        for price in (float("nan"), float("inf"), -float("inf")):
                            try:
                                client.order_insert("rb2601", "0", "0", 1, price, order_ref="1")
                                raise AssertionError("nonfinite order accepted")
                            except CTPError:
                                pass
                        assert client.qry_order() == []
                        after = client.qry_trading_account()
                        for field in ("Available", "FrozenMargin", "FrozenCommission", "CurrMargin"):
                            assert before[field] == after[field], field
                    admin.cmd("shutdown")
                output, _ = process.communicate(timeout=10)
                assert process.returncode == 0, output
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()
            journal = root / "journal"
            events = load_events(str(journal))
            assert [event["seq"] for event in events] == list(range(1, len(events) + 1))
            rebuild(str(journal), str(root / "projection.db"))
            if iteration:
                archives = list(root.glob("journal-run-*/journal"))
                assert len(archives) == 1
                assert load_events(str(archives[0])) == previous
                rebuild(str(archives[0]), str(root / "archive.db"))
            previous = events
    print("REVIEW REGRESSIONS: PASS")


if __name__ == "__main__":
    main()
