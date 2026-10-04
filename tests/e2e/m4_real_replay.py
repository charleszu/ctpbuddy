#!/usr/bin/env python
"""M4 controlled replay: one real ordinary-futures opening fill.

The source row is selected from broker exports at runtime and is never copied
into the repository. This proves the order/trade ABI path against real input
fields, not full account equivalence; close history, options and unavailable
reference fields remain outside this controlled fixture.
"""
from __future__ import annotations

import csv
import glob
import json
import os
import re
import socket
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))
from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402

BROKER = "8888"
EXPORT = os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export")
REFDATA = os.path.join(REPO, "refdata", "instruments.jsonl")


def free_port():
    s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); return p


def wait_port(port):
    for _ in range(200):
        try:
            socket.create_connection(("127.0.0.1", port), timeout=.2).close(); return
        except OSError: time.sleep(.05)
    raise AssertionError("port not ready")


def option_like(value):
    v = value.strip().upper()
    return bool(re.search(r"(?:-C-|[-_]C|C)[0-9]+$", v) or re.search(r"(?:-P-|[-_]P|P)[0-9]+$", v))


def choose_row():
    if not os.path.isdir(EXPORT):
        return None
    known = {json.loads(line)["instrument_id"] for line in open(REFDATA, encoding="utf-8")}
    for path in sorted(glob.glob(os.path.join(EXPORT, "*_trade.csv"))):
        orders_path = path.replace("_trade.csv", "_order.csv")
        orders = list(csv.DictReader(open(orders_path, encoding="utf-8-sig", newline="")))
        order_sys = {o.get("OrderSysID", "").strip() for o in orders}
        for row in csv.DictReader(open(path, encoding="utf-8-sig", newline="")):
            instrument = row["InstrumentID"].strip()
            if (instrument not in known or option_like(instrument) or
                    row.get("TradeType", "").strip() not in ("", "0") or
                    row.get("OffsetFlag", "").strip() not in ("0", "1")):
                continue
            if row.get("OrderSysID", "").strip() not in order_sys:
                continue
            return {k: row.get(k, "").strip() for k in ("TradingDay", "ExchangeID", "InstrumentID", "Direction", "OffsetFlag", "Price", "Volume", "OrderSysID", "TradeID")}
    return None


def scenario(path, row):
    price = float(row["Price"]); instrument = row["InstrumentID"]; exchange = row["ExchangeID"]
    cols = {c: "" for c in CANONICAL_COLUMNS}
    cols.update(instrument=instrument, exchange=exchange, trading_day=row["TradingDay"],
                update_time="09:00:01", last_price=str(price), volume="100",
                turnover=str(price), open_interest="1000", pre_settlement=str(price),
                settlement=str(price), pre_close=str(price), open=str(price), high=str(price),
                low=str(price), close=str(price), upper=str(price * 1.2), lower=str(price * .8),
                average=str(price), bid1=str(price - .01), ask1=str(price), bidvol1="10", askvol1="10")
    write_canonical(path, [cols])


def main():
    row = choose_row()
    if row is None:
        print("M4 REAL CONTROLLED REPLAY: SKIP (CTPBUDDY_EXPORT_DIR not provided or no eligible row)")
        return 0
    direction, offset = row["Direction"], row["OffsetFlag"]
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-real-replay-") as root:
        sdir = os.path.join(root, "scenario"); scenario(sdir, row)
        td, ap = free_port(), free_port()
        proc = subprocess.Popen([os.environ.get("CTPBUDDY_CORE", os.path.join(REPO, "core", "target", "debug", "ctpbuddy-server.exe")),
                                 "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}", "--broker-id", BROKER,
                                 "--data-dir", os.path.join(root, "data"), "--scenario", sdir], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        try:
            wait_port(ap)
            admin = Admin(f"127.0.0.1:{ap}"); admin.start_scenario(sdir, paused=True)
            cli = Client(f"127.0.0.1:{td}"); cli.auth(BROKER, "realreplay"); cli.login(BROKER, "realreplay"); cli.settle_confirm()
            admin.step()
            for _ in range(100):
                if admin.status()["playback"]["idx"] >= 1: break
                time.sleep(.02)
            cli.order_insert(row["InstrumentID"], direction=direction, offset=offset, volume=int(row["Volume"]),
                             limit_price=float(row["Price"]), exchange=row["ExchangeID"], order_ref="REAL-1")
            deadline = time.time() + 5; trades = []
            while time.time() < deadline:
                trades = [t for t in cli.qry_trade() if t["OrderRef"] == "REAL-1"]
                if trades: break
                time.sleep(.05)
            assert trades, row
            t = trades[0]
            assert t["InstrumentID"] == row["InstrumentID"]
            assert t["ExchangeID"] == row["ExchangeID"]
            assert t["Direction"] == ord(direction)
            assert t["OffsetFlag"] == ord(offset)
            assert t["Volume"] == int(row["Volume"])
            assert abs(t["Price"] - float(row["Price"])) < 1e-9
            print("REAL CONTROLLED REPLAY: PASS")
            print("source", row["TradingDay"], row["ExchangeID"], row["InstrumentID"], "volume", row["Volume"])
            print("verified", "direction/offset/price/volume/instrument/exchange")
            cli.close(); admin.shutdown(); proc.wait(timeout=5)
        finally:
            if proc.poll() is None: proc.kill(); proc.wait(timeout=5)
    return 0


if __name__ == "__main__": raise SystemExit(main())
