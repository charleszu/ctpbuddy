"""Record wall-clock budgets then replay them without real-time sleeps."""
from harness import TIME_SCALE
from pathlib import Path
import json
import socket
import sys
import tempfile
import time
import shutil

sys.path.insert(0, str(Path(__file__).resolve().parent))
from operations import server, make_scenario, RB, BROKER
from ctpbuddy.sdk import Admin, Client, CTPError
from ctpbuddy.wire import Frame, PING, PONG, AUTH, AUTH_RSP, ERR_RTN_ORDER_INSERT, REQ_QRY_TRADING_ACCOUNT
from ctpbuddy.replay import replay_journal, format_result
from ctpbuddy.journal import load_events


def main():
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-flow-replay-") as tmp:
        root = Path(tmp)
        scenario = root / "scenario"
        scenario.mkdir()
        make_scenario(str(scenario))
        with server(root, ["--ftd-max-comm-flux", "20", "--connect-freq", "1",
                           "--exchange-freq", "1", "--order-freq", "2", "--broker-id", "8888,9999"]) as (td, ap, _):
            with Admin(ap) as admin:
                admin.start_scenario(str(scenario), paused=True)
                try:
                    admin.cmd("flow_replay", data={})
                    raise AssertionError("offline replay command enabled in production")
                except RuntimeError as exc:
                    assert "offline" in str(exc)
                with Client(td) as cli:
                    cli._request(AUTH, json.dumps(dict(broker_id=BROKER, user_id="trace",
                        auth_code="do-not-record-this-secret")).encode(), expect=AUTH_RSP)
                    cli.login(BROKER, "trace", password="do-not-record-password")
                    cli.settle_confirm()
                    cli.subscribe([RB])
                    admin.step()
                    deadline = time.monotonic() + 3
                    while admin.status()["playback"]["idx"] < 1:
                        assert time.monotonic() < deadline
                        time.sleep(0.01)
                    # Send raw queries so the SDK's automatic retries do not
                    # smooth out the recorded over-budget requests.
                    cli.sock.sendall(b"".join(Frame(REQ_QRY_TRADING_ACCOUNT, n).encode() for n in (10, 11, 12)))
                    time.sleep(0.05)
                    cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, order_ref="1")
                    cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, order_ref="2")
                    late = cli.wait_late(ERR_RTN_ORDER_INSERT, timeout=0.5)
                    assert late is not None and CTPError.from_frame(late).error_id == 41
                    try:
                        cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, order_ref="3")
                        refused = cli.wait_late(ERR_RTN_ORDER_INSERT, timeout=0.5)
                        assert refused is not None and CTPError.from_frame(refused).error_id == 116
                    except CTPError as exc:
                        assert exc.error_id == 116
                    cli.order_action(RB, "1")  # insert and cancel have independent budgets
                    try:
                        cli.order_action(RB, "1")
                        raise AssertionError("cancel gate must reject the second action")
                    except CTPError as exc:
                        assert exc.error_id == 41
                    host, port = td.split(":")
                    with socket.create_connection((host, int(port)), timeout=2) as denied:
                        try:
                            denied.sendall(Frame(PING, 99).encode())
                            assert Frame.decode_from(denied) is None
                        except (ConnectionResetError, ConnectionAbortedError, BrokenPipeError):
                            pass
                    # Burst crosses the FTD window and saturates its bounded queue.
                    cli.sock.sendall(Frame(PING, 98, b"x" * (1 << 20)).encode())
                    cli.sock.sendall(b"".join(Frame(PING, n).encode() for n in range(100, 1160)))
                    time.sleep(1.15 * TIME_SCALE)
                    # Another investor starts in a new connection window.
                    with Client(td) as other:
                        other.auth(BROKER, "other")
                        other.login(BROKER, "other")
                        other.settle_confirm()
                        other.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, order_ref="1")
                    time.sleep(1.1 * TIME_SCALE)
                    with Client(td) as broker_peer:
                        broker_peer.auth("9999", "trace")
                        broker_peer.login("9999", "trace")
                        broker_peer.settle_confirm()
                        broker_peer.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3400, order_ref="1")
        journal = root / "data" / "journal"
        events = load_events(journal)
        assert any(e["type"] == "flow_input" for e in events)
        assert any(e["type"] == "flow_output" and e["data"]["req_id"] == 12 and e["data"]["error_id"] == 90 for e in events)
        assert any(e["type"] == "flow_output" and e["data"]["req_id"] >= 100 and e["data"]["error_id"] == 90 for e in events), "FTD queue saturation was not exercised"
        serialized = json.dumps(events)
        assert "do-not-record-this-secret" not in serialized
        assert "do-not-record-password" not in serialized
        raw = b"".join(bytes.fromhex(e["data"]["frame"]["payload"]) for e in events
                       if e["type"] == "flow_input" and "frame" in e["data"])
        assert b"do-not-record-this-secret" not in raw and b"do-not-record-password" not in raw
        result = replay_journal(str(journal), str(scenario), keep_data=True)
        print(format_result(result))
        assert result["ok"], result
        assert result["flow_replay"]["verified"]
        assert result["flow_replay"]["outputs"] > 0
        shutil.rmtree(result["data_dir"])
        # A changed callback ErrorID must fail even when the semantic core matches.
        changed = next(e for e in events if e["type"] == "flow_output" and e["data"]["error_id"] == 41)
        changed["data"]["error_id"] = 40
        altered = root / "altered.jsonl"
        altered.write_text("".join(json.dumps(e) + "\n" for e in events), encoding="utf-8")
        mismatched = replay_journal(str(altered), str(scenario))
        assert not mismatched["ok"] and not mismatched["flow_replay"]["verified"]
        print("FLOW REPLAY: PASS")


if __name__ == "__main__":
    main()
